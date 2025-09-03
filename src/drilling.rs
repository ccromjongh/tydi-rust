use crate::binary::TydiBinary;
use crate::TydiPacket;

pub trait TydiConvert<T> {
    fn convert(&self) -> Vec<TydiPacket<T>>;
}

impl<T: Clone> TydiConvert<T> for &[T] {
    fn convert(&self) -> Vec<TydiPacket<T>> {
        let len = self.len();
        self.iter().enumerate().map(|(i, el)| TydiPacket { data: Some((*el).clone()), last: vec![i == len-1] }).collect()
    }
}

impl<T: Clone> TydiConvert<T> for Vec<T> {
    fn convert(&self) -> Vec<TydiPacket<T>> {
        let len = self.len();
        self.iter().enumerate().map(|(i, el)| TydiPacket { data: Some((*el).clone()), last: vec![i == len-1] }).collect()
    }
}

pub trait TydiDrill<T: Clone, B> {
    fn drill<F>(&self, f: F) -> Vec<TydiPacket<<B as IntoIterator>::Item>>
    where
        F: Fn(T) -> B,
        B: IntoIterator;
}

impl<T: Clone, B> TydiDrill<T, B> for Vec<TydiPacket<T>> {
    fn drill<F>(&self, f: F) -> Vec<TydiPacket<<B as IntoIterator>::Item>>
    where
        F: Fn(T) -> B,
        B: IntoIterator
    {
        type ResultType<B> = Vec<TydiPacket<<B as IntoIterator>::Item>>;

        let d = self.first().and_then(|el| Some(el.last.len())).unwrap_or(0);
        // Map through existing items in our vector of packets
        self.iter().flat_map(|el| {
            let el = (*el).clone();
            let new_lasts = [el.last.clone(), vec![false]].concat();
            // If the packet contains data
            let new_vec: ResultType<B> = if let Some(old_data) = el.data {
                // Apply drilling function create packets from elements of resulting iterable
                let mut res: ResultType<B> = f(old_data).into_iter().map(|n_el| {
                    TydiPacket { data: Some(n_el), last: new_lasts.clone() }
                }).collect();

                // It can be that this dimension is empty, in that case return a single empty packet
                if res.is_empty() {
                    vec![TydiPacket { data: None, last: [el.last.clone(), vec![true]].concat() }]
                } else {
                    // Patch last element
                    /*if let Some(el) = res.last_mut() {
                        el.last[d] = true
                    }*/
                    let res_len = res.len();
                    res[res_len - 1].last[d] = true;
                    res
                }
            } else {
                vec![TydiPacket {
                    data: None,
                    last: new_lasts,
                }]
            };
            new_vec
        }).collect()
    }
}

pub trait TydiPacktestToBinary {
    fn finish(&self, size: usize) -> Vec<TydiBinary>;
}

impl<T: Into<TydiBinary> + Clone> TydiPacktestToBinary for Vec<TydiPacket<T>> {
    fn finish(&self, size: usize) -> Vec<TydiBinary> {
        self.iter().map(|el| el.clone().to_binary(size)).collect()
    }
}

use crate::TydiPacket;

pub trait TydiConvert<T> {
    fn convert(&self) -> Vec<TydiPacket<&T>>;
}

impl<T: Clone> TydiConvert<T> for &[T] {
    fn convert(&self) -> Vec<TydiPacket<&T>> {
        let len = self.len();
        self.iter().enumerate().map(|(i, el)| TydiPacket { data: Some(el), last: vec![i == len-1] }).collect()
    }
}

pub trait TydiDrill<T, B> {
    fn drill<F>(&self, f: F) -> Vec<TydiPacket<<B as IntoIterator>::Item>>
    where
        F: Fn(&T) -> B,
        B: IntoIterator;
}

impl<T, B> TydiDrill<T, B> for Vec<TydiPacket<&T>> {
    fn drill<F>(&self, f: F) -> Vec<TydiPacket<<B as IntoIterator>::Item>>
    where
        F: Fn(&T) -> B,
        B: IntoIterator
    {
        self.iter().flat_map(|el| {
            let new_vec = if let Some(old_data) = el.data {
                f(old_data).into_iter().enumerate().map(|(j, n_el)| {
                    let len = self.len(); // Fixme should be inner length
                    // let mut new_lasts = el.last.clone();
                    // new_lasts.push(j == len - 1);
                    let new_lasts = [el.last.clone(), vec![j == len - 1]].concat();
                    TydiPacket { data: Some(n_el), last: new_lasts }
                }).collect::<Vec<_>>()
            } else {
                let new_lasts = [el.last.clone(), vec![false]].concat();
                vec![TydiPacket {
                    data: None,
                    last: new_lasts,
                }]
            };
            new_vec
        }).collect()
    }
}

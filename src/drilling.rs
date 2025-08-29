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
                let mut it = f(old_data).into_iter().peekable();
                let mut res: Vec<TydiPacket<_>> = Vec::new();
                while let Some(n_el) = it.next() {
                    let is_last = it.peek().is_none();
                    let new_lasts = [el.last.clone(), vec![is_last]].concat();
                    res.push(TydiPacket { data: Some(n_el), last: new_lasts })
                }
                res
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

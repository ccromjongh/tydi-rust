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
    fn drill<F>(&self, f: F) -> Vec<TydiPacket<B>>
    where
        F: Fn(&T) -> B;
}

impl<T, B> TydiDrill<T, B> for Vec<TydiPacket<&T>> {
    fn drill<F>(&self, f: F) -> Vec<TydiPacket<B>>
    where
        F: Fn(&T) -> B
    {
        let len = self.len();

        self.iter().enumerate().map(|(i, el)| {
            let mut new_lasts = el.last.clone();
            new_lasts.push(i == len - 1);
            let new_data: Option<B> = el.data.and_then(|e| Some(f(e)));
            TydiPacket {
                data: new_data,
                last: new_lasts,
            }
        }).collect()
    }
}

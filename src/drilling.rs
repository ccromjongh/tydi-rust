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

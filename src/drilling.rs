use crate::binary::TydiBinary;
use crate::{binary, TydiPacket};

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

pub fn packets_from_binaries<T: binary::FromTydiBinary>(value: Vec<TydiBinary>, dim: usize) -> Vec<TydiPacket<T>> {
    value.iter().map(|el| TydiPacket::from_binary(el.clone(), dim)).collect()
}

pub trait TydiDrill<T: Clone> {
    /// "Drill" into the structure to the iterable field referenced in [f], creating a new dimension in the `last` data.
    fn drill<F, B>(&self, f: F) -> Vec<TydiPacket<<B as IntoIterator>::Item>>
    where
        F: Fn(T) -> B,
        B: IntoIterator;

    /// Inject the [data] in the vector referenced in the function [f] by consuming the lowest dimension in the `last` data.
    fn inject<F, B>(& mut self, f: F, data: Vec<TydiPacket<B>>) -> &mut Self
    where
        F: Fn(&mut T) -> &mut Vec<B>,
        B: Clone;

    /// Creates one layer of `Vec` by consuming the lowest dimension in the `last` data.
    fn vectorize(self) -> Vec<Vec<TydiPacket<T>>>;

    /// Creates one layer of `Vec` inside the packet by consuming the lowest dimension in the `last` data.
    fn vectorize_inner(self) -> Vec<TydiPacket<Vec<T>>>;
}

impl<T: Clone> TydiDrill<T> for Vec<TydiPacket<T>> {
    fn drill<F, B>(&self, f: F) -> Vec<TydiPacket<<B as IntoIterator>::Item>>
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

    fn inject<F, B>(&mut self, f: F, data: Vec<TydiPacket<B>>) -> &mut Self
    where
        F: Fn(&mut T) -> &mut Vec<B>,
        B: Clone
    {
        let mut data_iter = data.iter();
        for x in self.iter_mut() {
            let self_option = x.data.as_mut();
            if self_option.is_none() {
                data_iter.next();
                continue
            }
            let self_data = self_option.unwrap();
            let mut target = f(self_data);
            while let Some(el) = data_iter.next() {
                if el.data.is_none() { break }
                target.push(el.data.clone().unwrap());
                if *el.last.last().unwrap() == true {
                    break
                }
            }
        }
        self
    }

    fn vectorize(self) -> Vec<Vec<TydiPacket<T>>> {
        let mut result: Vec<Vec<TydiPacket<T>>> = Vec::new();
        let mut inner_result: Vec<TydiPacket<T>> = Vec::new();
        for x in self.iter() {
            let mut last_copy = x.last.clone();
            let last_el_in_dim = last_copy.pop().unwrap();
            // If the element is the last in the lowest dimension and empty (None) we don't push the item
            if !(last_el_in_dim && x.data.is_none()) {
                inner_result.push(TydiPacket {
                    data: x.data.clone(),
                    last: last_copy,
                });
            }
            if last_el_in_dim {
                result.push(inner_result);
                inner_result = Vec::new();
            }
        }
        result
    }

    fn vectorize_inner(self) -> Vec<TydiPacket<Vec<T>>> {
        // The top vector gets shorter as items are placed in the inner vectors instead.
        let mut result: Vec<TydiPacket<Vec<T>>> = Vec::new();
        let mut inner_result: Vec<T> = Vec::new();
        for x in self.iter() {
            let mut last_copy = x.last.clone();
            let last_el_in_dim = last_copy.pop().unwrap();
            if x.data.is_some() {
                inner_result.push(x.data.clone().unwrap());
            }
            if last_el_in_dim {
                result.push(TydiPacket {
                    data: Some(inner_result),
                    last: last_copy,
                });
                inner_result = Vec::new();
            } else if x.data.is_none() {
                result.push(TydiPacket {
                    data: None,
                    last: last_copy,
                });
                inner_result = Vec::new();
            }
        }
        result
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

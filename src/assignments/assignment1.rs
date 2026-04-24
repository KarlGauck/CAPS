use std::fmt::Debug;
use plotters::prelude::*;
use num_traits::{Float, NumCast};

pub fn ex1() {
    sumHelper::<f32>(true);
}

fn sumHelper<T: Float + Debug>(reverse: bool) {
    const SINF: f32 = std::f32::consts::PI;
    let ks = [1000, 100000, 10000000, 100000000];


    for k in ks {
        let vec: Vec<T> = if reverse {
            (1..k).rev().map(|x| NumCast::from(x).unwrap()).collect()
        } else {
            (1..k).map(|x| NumCast::from(x).unwrap()).collect()
        };
        println!("{:?}", sum::<T>(&vec));
    }
}

pub fn sum<T: Float>(list: &Vec<T>) -> T {
    let mut sum: T = T::zero();
    for &i in list {
        sum = sum + T::one()/(i*i)
    }
    sum
}
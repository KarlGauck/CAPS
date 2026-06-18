struct RC3BSystem {
    mu1: f64,
    mu2: f64,
    r1: f64,
    r2: f64,
    angular_momentum: f64
}

impl RC3BSystem {
    fn initial_y(&mut self, jacobi_constant: f64, initial_x: f64) -> f64 {
        (self.angular_momentum.powi(2) * initial_x.powi(2) + 2*(self.mu1/self.r1) + (self.mu2/self.r2) - jacobi_constant).sqrt()
    }
}


fn rk4<T>()
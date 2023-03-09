use core::fmt::Debug;
use crate::system::params::*;
use csv::Writer;
use crate::system::consts::*;


#[derive(Debug, Clone, Copy)]
pub struct Stat {
    pub KE: f32,
    pub PE: f32,
}
unsafe impl bytemuck::Pod for Stat {}
unsafe impl bytemuck::Zeroable for Stat {}

impl Stat {
    pub fn new() -> Self {
        Self {
            KE: 0.0,
            PE: 0.0,
        }
    }

    pub fn create_stats(N: usize) -> Vec<Stat> {
        let mut stats = Vec::new();
        for _ in 0..N {
            stats.push(Stat::new());
        }
        stats
    }

    pub fn size() -> wgpu::BufferAddress {
        std::mem::size_of::<Stat>() as wgpu::BufferAddress
    }

    pub fn desc(binding: u32, num_particles: u64, read_only: bool) -> wgpu::BindGroupLayoutEntry {
        wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Storage { read_only },
                has_dynamic_offset: false,
                min_binding_size: wgpu::BufferSize::new(Stat::size() * num_particles),
            },
            count: None,
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Stats {
    pub iteration: usize,
    pub KE: f32,
    pub PE: f32,
}


impl Debug for Stats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Stats")
            .field("iteration", &self.iteration)
            .field("KE", &self.KE)
            .field("PE", &self.PE)
            .finish()
    }
}

pub struct StatHistory {
    params: Params,
    itaration: Vec<usize>,
    KE: Vec<f32>,
    PE: Vec<f32>,
}

impl StatHistory {
    pub fn new(params: Params) -> Self {
        Self {
            params,
            itaration: Vec::new(),
            KE: Vec::new(),
            PE: Vec::new(),
        }
    }

    pub fn add(&mut self, stats: Stats) {
        self.itaration.push(stats.iteration);
        self.KE.push(stats.KE);
        self.PE.push(stats.PE);
    }

    fn sort(&mut self) {
        // sort by iteration
        let mut vec = Vec::new();
        for index in 0..self.itaration.len() {
            vec.push((self.itaration[index], self.KE[index], self.PE[index]));
        }
        vec.sort_by(|a, b| a.0.cmp(&b.0));
        self.itaration.clear();
        self.KE.clear();
        self.PE.clear();
        for index in 0..vec.len() {
            self.itaration.push(vec[index].0);
            self.KE.push(vec[index].1);
            self.PE.push(vec[index].2);
        }
    }

    pub fn save(&mut self, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
        // self.sort();
        if self.itaration.len() == 0 {
            println!("error: itaration.len() == 0");
            return Ok(());
        }

        let mut wtr = Writer::from_path(filename)?;
        // header data
        wtr.write_record(&[self.params.to_string(), "".to_string(), "".to_string()])?;
        wtr.write_record(&["iteration", "KE", "PE"])?;
        // data
        for index in 0..self.itaration.len() {
            wtr.write_record(&[
                self.itaration[index].to_string(),
                self.KE[index].to_string(),
                self.PE[index].to_string(),
            ])?;
        }
        wtr.flush()?;
        Ok(())
    }

    pub fn clone(&self) -> Self {
        Self {
            params: self.params,
            itaration: self.itaration.clone(),
            KE: self.KE.clone(),
            PE: self.PE.clone(),
        }
    }

    pub fn graph_KE(&self, sample_rate: usize) -> Vec<[f64; 2]> {
        
        let mut graph = Vec::new();
        let iter = self.itaration.len()/sample_rate;
        for index in 0..iter {
            graph.push([self.itaration[index*sample_rate] as f64, self.KE[index*sample_rate] as f64]);
        }
        graph
    }

    pub fn graph_PE(&self, sample_rate: usize) -> Vec<[f64; 2]> {
        let mut graph = Vec::new();
        let iter = self.itaration.len()/sample_rate;
        for index in 0..iter {
            graph.push([self.itaration[index*sample_rate] as f64, self.PE[index*sample_rate] as f64]);
        }
        graph
    }

    pub fn graph_TE(&self, sample_rate: usize) -> Vec<[f64; 2]> {
        let mut graph = Vec::new();
        let iter = self.itaration.len()/sample_rate;
        for index in 0..iter {
            if index*sample_rate >= self.itaration.len() {
                break;
            }
            graph.push([self.itaration[index*sample_rate] as f64, (self.KE[index*sample_rate] + self.PE[index*sample_rate]) as f64]);
        }
        graph
    }

    pub fn temperature(&self) -> f32 {
        // get last temperature ½m<v2> = (3/2)kBT
        if self.itaration.len() == 0 {
            return 0.0;
        }
        let index = self.itaration.len() - 1;
        let v2 = self.KE[index] / self.params.N as f32;
        let kBT = v2 * 2.0 / 3.0 / BOLTZMANN_CONSTANT_EV;
        kBT
    }

    pub fn velocity_rms(&self) -> f32 {
        // get last temperature ½m<v2> = (3/2)kBT
        if self.itaration.len() == 0 {
            return 0.0;
        }
        // let index = self.itaration.len() - 1;
        // let v2 = self.KE[index] / self.params.N as f32 * 2.0 / self.params.helium.mass;
        // let v = v2.sqrt();  // in nm/ps
        // v * 1e-3 // in m/s
        (self.temperature() * 3.0 * BOLTZMANN_CONSTANT_J / (self.params.helium.mass * 1.66053906660e-27)).sqrt()
    }
}
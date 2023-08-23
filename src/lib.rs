#![feature(test)]
use std::collections::HashMap;
use std::fs;
use std::path::Path;

type CpuFreqError = Box<dyn std::error::Error>;
pub trait CpuFreq {
    const CPUFREQ_PATH: &'static str;
    fn read_file(fname: &str) -> Result<String, std::io::Error> {
        let path = Path::new(Self::CPUFREQ_PATH).join(fname);
        Ok(String::from_utf8(fs::read(path)?).unwrap())
    }
    fn write_file(fname: &str, data: &str) -> Result<(), std::io::Error> {
        let path = Path::new(Self::CPUFREQ_PATH).join(fname);
        fs::write(path, data)
    }
    fn get_ranges(fname: &str) -> Result<Vec<usize>, std::io::Error> {
        let range = Self::read_file(fname)?;
        let mut l: Vec<usize> = Vec::new();
        for r in range.split(",") {
            let mr: Vec<usize> = r.split("-").map(|x| x.trim().parse().unwrap()).collect();
            if mr.len() == 2 {
                l.extend(mr[0]..=mr[1]);
            } else {
                l.push(mr[0]);
            }
        }
        Ok(l)
    }
    fn get_variable<T>(id: usize, var: &str) -> Result<T, CpuFreqError>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + 'static,
    {
        let path = format!("cpu{id}/cpufreq/{var}");
        let cpu_data = Self::read_file(&path)?;
        Ok(cpu_data.trim().parse()?)
    }
    fn set_variable(id: usize, var: &str, data: &str) -> Result<(), std::io::Error> {
        let path = format!("cpu{id}/cpufreq/{var}");
        Self::write_file(&path, data)
    }

    fn get_variable_all<T>(var: &str) -> Result<HashMap<usize, T>, CpuFreqError>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + 'static,
    {
        let mut data = HashMap::new();
        for cpu in Self::get_ranges("online")? {
            data.insert(cpu, Self::get_variable(cpu, var)?);
        }
        Ok(data)
    }
    fn set_variable_all(var: &str, data: &str) -> Result<(), std::io::Error> {
        for cpu in Self::get_ranges("online")? {
            let path = format!("cpu{cpu}/cpufreq/{var}");
            Self::write_file(&path, data)?
        }
        Ok(())
    }
}

struct CPU {}

impl CpuFreq for CPU {
    const CPUFREQ_PATH: &'static str = "/sys/devices/system/cpu/";
}

impl CPU {
    fn new() -> Result<Self, std::io::Error> {
        Ok(CPU {})
    }
    fn online(&self) -> Result<Vec<usize>, CpuFreqError> {
        Ok(CPU::get_ranges("online")?)
    }
    fn governors(&self) -> Result<HashMap<usize, String>, CpuFreqError> {
        Ok(CPU::get_variable_all("scaling_governor")?)
    }
    fn frequencies(&self) -> Result<HashMap<usize, u64>, CpuFreqError> {
        Ok(CPU::get_variable_all("scaling_cur_freq")?)
    }
    fn max_frequencies(&self) -> Result<HashMap<usize, u64>, CpuFreqError> {
        Ok(CPU::get_variable_all("scaling_cur_freq")?)
    }
    fn min_frequencies(&self) -> Result<HashMap<usize, u64>, CpuFreqError> {
        Ok(CPU::get_variable_all("scaling_cur_freq")?)
    }
    fn available_frequencies(&self) -> Result<HashMap<usize, Vec<u64>>, CpuFreqError> {
        let mut res = HashMap::new();
        for (cpu, freq) in CPU::get_variable_all::<String>("scaling_available_frequencies")? {
            res.insert(cpu, freq.split(" ").map(|x| x.parse().unwrap()).collect());
        }
        Ok(res)
    }
    fn set_frequencies<T: ToString>(freq: T) -> Result<(), CpuFreqError> {
        CPU::set_variable_all("scaling_setspeed", &freq.to_string())?;
        CPU::set_variable_all("scaling_max_freq", &freq.to_string())?;
        CPU::set_variable_all("scaling_min_freq", &freq.to_string())?;
        Ok(())
    }
    fn set_governors(gov: &str) -> Result<(), CpuFreqError> {
        Ok(CPU::set_variable_all("scaling_governor", &gov)?)
    }
    fn enable(id: usize) -> Result<(), CpuFreqError> {
        Ok(CPU::set_variable(id, "online", "1")?)
    }
    fn disable(id: usize) -> Result<(), CpuFreqError> {
        Ok(CPU::set_variable(id, "online", "0")?)
    }
    fn enable_all() -> Result<(), CpuFreqError> {
        for cpu in CPU::get_ranges("present")? {
            Self::enable(cpu)?;
        }
        Ok(())
    }
    fn disable_all() -> Result<(), CpuFreqError> {
        for cpu in CPU::get_ranges("present")? {
            Self::disable(cpu)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    extern crate test;
    use test::Bencher;

    macro_rules! test_method {
        ($method: ident) => {
            #[bench]
            fn $method(b: &mut Bencher) {
                b.iter(|| {
                    let cpu = CPU::new().unwrap();
                    for v in cpu.$method().unwrap() {}
                });
            }
        };
    }

    test_method!(online);
    test_method!(governors);
    test_method!(frequencies);
    test_method!(max_frequencies);
    test_method!(min_frequencies);
    test_method!(available_frequencies);
}

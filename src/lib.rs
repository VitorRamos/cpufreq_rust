#![feature(test)]
use std::collections::{HashMap, HashSet};
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
    fn new() -> Result<Self, CpuFreqError> {
        if std::env::consts::OS != "linux" {
            let err =
                std::io::Error::new(std::io::ErrorKind::Unsupported, "Only supported on Linux");
            return Err(Box::new(err));
        }
        let driver: String = Self::get_variable(0, "scaling_driver")?;
        match driver.as_str() {
            "acpi-cpufreq" => {}
            "intel-pstate" => {}
            _ => {
                let err =
                    std::io::Error::new(std::io::ErrorKind::Unsupported, "Only supported driver");
                return Err(Box::new(err));
            }
        };
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
    fn set_frequencies<T: ToString>(&self, freq: T) -> Result<(), CpuFreqError> {
        CPU::set_variable_all("scaling_setspeed", &freq.to_string())?;
        CPU::set_variable_all("scaling_max_freq", &freq.to_string())?;
        CPU::set_variable_all("scaling_min_freq", &freq.to_string())?;
        Ok(())
    }
    fn set_max_frequencies<T: ToString>(&self, freq: T) -> Result<(), CpuFreqError> {
        CPU::set_variable_all("scaling_max_freq", &freq.to_string())?;
        Ok(())
    }
    fn set_min_frequencies<T: ToString>(&self, freq: T) -> Result<(), CpuFreqError> {
        CPU::set_variable_all("scaling_min_freq", &freq.to_string())?;
        Ok(())
    }
    fn set_governors(&self, gov: &str) -> Result<(), CpuFreqError> {
        Ok(CPU::set_variable_all("scaling_governor", &gov)?)
    }
    fn enable(&self, id: usize) -> Result<(), CpuFreqError> {
        Ok(CPU::write_file(&format!("cpu{id}/online"), "1")?)
    }
    fn disable(&self, id: usize) -> Result<(), CpuFreqError> {
        Ok(CPU::write_file(&format!("cpu{id}/online"), "0")?)
    }
    fn enable_all(&self) -> Result<(), CpuFreqError> {
        for cpu in CPU::get_ranges("present")? {
            if cpu != 0 {
                self.enable(cpu)?;
            }
        }
        Ok(())
    }
    fn disable_all(&self) -> Result<(), CpuFreqError> {
        for cpu in CPU::get_ranges("present")? {
            if cpu != 0 {
                self.disable(cpu)?;
            }
        }
        Ok(())
    }
    fn disable_hyperthread(&self) -> Result<(), CpuFreqError> {
        let mut to_disable = HashSet::new();
        for cpu in CPU::get_ranges("online")? {
            let path = format!("cpu{cpu}/topology/thread_siblings_list");
            let cpu_data = Self::get_ranges(&path)?;
            to_disable.insert(cpu_data[1]);
        }
        for cpu in to_disable {
            self.disable(cpu)?;
        }
        Ok(())
    }
    fn reset(&self) -> Result<(), CpuFreqError> {
        self.enable_all()?;
        self.set_governors("schedutil")?;
        let avail_freqs = self.available_frequencies()?;
        let max_freq = avail_freqs.get(&0).unwrap().iter().max().unwrap();
        let min_freq = avail_freqs.get(&0).unwrap().iter().min().unwrap();
        self.set_max_frequencies(max_freq)?;
        self.set_min_frequencies(min_freq)?;
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
                    for _ in cpu.$method().unwrap() {}
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

    #[test]
    fn disable() {
        let cpu = CPU::new().unwrap();
        cpu.enable_all().unwrap();
        let online_before = cpu.online().unwrap();
        cpu.disable(1).unwrap();
        cpu.disable(4).unwrap();
        let online_after = cpu.online().unwrap();
        assert!(
            online_after.len() < online_before.len(),
            "{} should be less than {}",
            online_after.len(),
            online_before.len()
        );
        let mut x: Vec<&usize> = online_before
            .iter()
            .filter(|x| !online_after.contains(x))
            .collect();
        x.sort();
        assert_eq!(x.len(), 2);
        assert_eq!(*x[0], 1);
        assert_eq!(*x[1], 4);
        cpu.enable_all().unwrap();
    }
    #[test]
    fn enable() {
        let cpu = CPU::new().unwrap();
        cpu.disable_all().unwrap();
        let online_before = cpu.online().unwrap();
        cpu.enable(1).unwrap();
        cpu.enable(4).unwrap();
        let online_after = cpu.online().unwrap();
        assert!(
            online_after.len() > online_before.len(),
            "{} should be less than {}",
            online_after.len(),
            online_before.len()
        );
        let mut x: Vec<usize> = cpu.online().unwrap();
        x.sort();
        assert_eq!(x.len(), 3);
        assert_eq!(x[0], 0);
        assert_eq!(x[1], 1);
        assert_eq!(x[2], 4);
        cpu.enable_all().unwrap();
    }
    #[test]
    fn hyperthread() {
        let cpu = CPU::new().unwrap();
        cpu.disable_hyperthread().unwrap();
    }
    #[test]
    fn reset() {
        let cpu = CPU::new().unwrap();
        cpu.reset().unwrap();
    }
}

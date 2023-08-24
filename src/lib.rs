//! Wrapper arround the cpufreq fs
#![feature(test)]
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Cpufreq error type
type CpuFreqError = Box<dyn std::error::Error>;

/// Base cpufreq functionality for reading and writing on cpu variables
pub trait CpuFreq {
    // Base path to be defined
    const CPUFREQ_PATH: &'static str;
    /// Read file on CPUFREQ_PATH
    ///
    /// # Panics
    ///
    /// Panics if the content of the file is not utf8 encoded
    fn read_file(fname: &str) -> Result<String, std::io::Error> {
        let path = Path::new(Self::CPUFREQ_PATH).join(fname);
        Ok(String::from_utf8(fs::read(path)?).unwrap())
    }
    /// Write to file on CPUFREQ_PATH
    fn write_file(fname: &str, data: &str) -> Result<(), std::io::Error> {
        let path = Path::new(Self::CPUFREQ_PATH).join(fname);
        fs::write(path, data)
    }
    /// Read and parse cpufreq ranges, example:
    /// 0,4,6-12,18 -> \[0,4,6,7,8,9,10,12,18\]
    ///
    /// # Panics
    /// Panics if values cannot be parsed to usize
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
    /// Read a specific variable and parse to T
    fn get_variable<T>(id: usize, var: &str) -> Result<T, CpuFreqError>
    where
        T: std::str::FromStr,
        T::Err: std::error::Error + 'static,
    {
        let path = format!("cpu{id}/cpufreq/{var}");
        let cpu_data = Self::read_file(&path)?;
        Ok(cpu_data.trim().parse()?)
    }
    ///  Sets a specific variable
    fn set_variable(id: usize, var: &str, data: &str) -> Result<(), std::io::Error> {
        let path = format!("cpu{id}/cpufreq/{var}");
        Self::write_file(&path, data)
    }
    /// Get variables for all online cpus
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
    /// Set variables for all online cpus
    fn set_variable_all(var: &str, data: &str) -> Result<(), std::io::Error> {
        for cpu in Self::get_ranges("online")? {
            let path = format!("cpu{cpu}/cpufreq/{var}");
            Self::write_file(&path, data)?
        }
        Ok(())
    }
}

/// CPU object
pub struct CPU {}

impl CpuFreq for CPU {
    /// Base path for cpufreq
    const CPUFREQ_PATH: &'static str = "/sys/devices/system/cpu/";
}

impl CPU {
    /// Creates a new CPU
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.frequencies().expect("Unable to read frequencies");
    /// ```
    pub fn new() -> Result<Self, CpuFreqError> {
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
    /// Get online cpus
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.online().expect("Unable to read online cpus");
    /// ```
    pub fn online(&self) -> Result<Vec<usize>, CpuFreqError> {
        Ok(CPU::get_ranges("online")?)
    }
    /// Get online governors
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.governors().expect("Unable to read online governors");
    /// ```
    pub fn governors(&self) -> Result<HashMap<usize, String>, CpuFreqError> {
        Ok(CPU::get_variable_all("scaling_governor")?)
    }
    /// Get online frequencies
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.frequencies().expect("Unable to read online frequencies");
    /// ```
    pub fn frequencies(&self) -> Result<HashMap<usize, u64>, CpuFreqError> {
        Ok(CPU::get_variable_all("scaling_cur_freq")?)
    }
    /// Get online max_frequencies
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.max_frequencies().expect("Unable to read online max_frequencies");
    /// ```
    pub fn max_frequencies(&self) -> Result<HashMap<usize, u64>, CpuFreqError> {
        Ok(CPU::get_variable_all("scaling_cur_freq")?)
    }
    /// Get online min_frequencies
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.min_frequencies().expect("Unable to read online min_frequencies");
    /// ```
    pub fn min_frequencies(&self) -> Result<HashMap<usize, u64>, CpuFreqError> {
        Ok(CPU::get_variable_all("scaling_cur_freq")?)
    }
    /// Get online min_frequencies
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.min_frequencies().expect("Unable to read online min_frequencies");
    /// ```
    pub fn available_frequencies(&self) -> Result<HashMap<usize, Vec<u64>>, CpuFreqError> {
        let mut res = HashMap::new();
        for (cpu, freq) in CPU::get_variable_all::<String>("scaling_available_frequencies")? {
            res.insert(cpu, freq.split(" ").map(|x| x.parse().unwrap()).collect());
        }
        Ok(res)
    }
    /// Set online cpu frequencies
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// cpu.set_governors("userspace");
    /// let freqs = cpu.set_frequencies("2300000").expect("Unable to set frequencies");
    /// ```
    pub fn set_frequencies<T: ToString>(&self, freq: T) -> Result<(), CpuFreqError> {
        CPU::set_variable_all("scaling_setspeed", &freq.to_string())?;
        CPU::set_variable_all("scaling_max_freq", &freq.to_string())?;
        CPU::set_variable_all("scaling_min_freq", &freq.to_string())?;
        Ok(())
    }
    /// Set online cpu max possible frequencies
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.set_max_frequencies(2301000).expect("Unable to set max frequencies");
    /// ```
    pub fn set_max_frequencies<T: ToString>(&self, freq: T) -> Result<(), CpuFreqError> {
        CPU::set_variable_all("scaling_max_freq", &freq.to_string())?;
        Ok(())
    }
    /// Set online cpu min possible frequencies
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.set_min_frequencies(2301000).expect("Unable to set min frequencies");
    /// ```
    pub fn set_min_frequencies<T: ToString>(&self, freq: T) -> Result<(), CpuFreqError> {
        CPU::set_variable_all("scaling_min_freq", &freq.to_string())?;
        Ok(())
    }
    /// Set online cpu governors
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.set_governors("ondemand").expect("Unable to set governors");
    /// ```
    pub fn set_governors(&self, gov: &str) -> Result<(), CpuFreqError> {
        Ok(CPU::set_variable_all("scaling_governor", &gov)?)
    }
    /// Enable one cpu
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.enable(5).expect("Unable enable cpu");
    /// ```
    pub fn enable(&self, id: usize) -> Result<(), CpuFreqError> {
        Ok(CPU::write_file(&format!("cpu{id}/online"), "1")?)
    }
    /// Disable one cpu
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.disable(5).expect("Unable disable cpu");
    /// ```
    pub fn disable(&self, id: usize) -> Result<(), CpuFreqError> {
        Ok(CPU::write_file(&format!("cpu{id}/online"), "0")?)
    }
    /// Enable all present cpus
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.enable_all().expect("Unable to enable all present cpus");
    /// ```
    pub fn enable_all(&self) -> Result<(), CpuFreqError> {
        for cpu in CPU::get_ranges("present")? {
            if cpu != 0 {
                self.enable(cpu)?;
            }
        }
        Ok(())
    }
    /// Disable all present cpus
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.disable_all().expect("Unable to disable all present cpus");
    /// ```
    pub fn disable_all(&self) -> Result<(), CpuFreqError> {
        for cpu in CPU::get_ranges("present")? {
            if cpu != 0 {
                self.disable(cpu)?;
            }
        }
        Ok(())
    }
    /// Disable all siblings threads
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// cpu.enable_all();
    /// let freqs = cpu.disable_hyperthread().expect("Unable to disable hyperthread");
    /// ```
    pub fn disable_hyperthread(&self) -> Result<(), CpuFreqError> {
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
    /// Reset cpu governor, max and min frequencies
    ///
    /// # Example
    /// ```
    /// use cpufreq::CPU;
    ///
    /// let cpu = CPU::new().unwrap();
    /// let freqs = cpu.reset().expect("Unable to reset cpu");
    /// ```
    pub fn reset(&self) -> Result<(), CpuFreqError> {
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
        cpu.enable_all().unwrap();
        cpu.disable_hyperthread().unwrap();
    }
    #[test]
    fn reset() {
        let cpu = CPU::new().unwrap();
        cpu.reset().unwrap();
    }
}

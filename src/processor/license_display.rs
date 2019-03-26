use std::process;

use crate::config::{Config, SelectedLicenses};
use crate::processor::ProcessWithConfig;

const SIC_LICENSE: &str = include_str!("../../LICENSE");
const DEP_LICENSES: &str = include_str!("../../LICENSES_DEPENDENCIES");

#[cfg(not(windows))]
const NEW_LINE: &'static str = "\n";
#[cfg(windows)]
const NEW_LINE: &'static str = "\r\n";

#[derive(Debug, Default)]
pub struct LicenseDisplayProcessor;

impl LicenseDisplayProcessor {
    fn print_licenses(slice: &[SelectedLicenses], tool_name: &str) {
        for item in slice {
            match item {
                SelectedLicenses::ThisSoftware => {
                    println!(
                        "{} image tools license:{}{}{}{}{}",
                        tool_name, NEW_LINE, NEW_LINE, SIC_LICENSE, NEW_LINE, NEW_LINE
                    );
                }
                SelectedLicenses::Dependencies => println!("{}", DEP_LICENSES),
            };
        }

        if !slice.is_empty() {
            process::exit(0);
        }
    }
}

impl ProcessWithConfig<()> for LicenseDisplayProcessor {
    fn process(&self, config: &Config) {
        LicenseDisplayProcessor::print_licenses(&config.licenses, &config.tool_name);
    }
}

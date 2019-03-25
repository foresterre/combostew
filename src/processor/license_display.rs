use std::process;

use crate::config::{Config, SelectedLicenses};
use crate::processor::ProcessWithConfig;

const SIC_LICENSE: &str = include_str!("../../LICENSE");
const DEP_LICENSES: &str = include_str!("../../LICENSES_DEPENDENCIES");

#[derive(Debug, Default)]
pub struct LicenseDisplayProcessor;

impl LicenseDisplayProcessor {
    fn print_licenses(slice: &[SelectedLicenses], tool_name: &str) {
        for item in slice {
            match item {
                SelectedLicenses::ThisSoftware => {
                    println!("{} image tools license:\n\n{}\n\n", tool_name, SIC_LICENSE);
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

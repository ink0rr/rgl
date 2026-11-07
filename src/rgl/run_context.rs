use super::{Config, Temp};

pub struct RunContext<'a> {
    pub config: &'a Config,
    pub temp: &'a Temp,
    pub profile: &'a str,
    pub extra_args: &'a [String],
}

impl<'a> RunContext<'a> {
    pub fn new(
        config: &'a Config,
        temp: &'a Temp,
        profile: &'a str,
        extra_args: &'a [String],
    ) -> Self {
        Self {
            config,
            temp,
            profile,
            extra_args,
        }
    }
}

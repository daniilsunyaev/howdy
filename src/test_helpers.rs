#[cfg(test)]
pub fn build_cli_args(args_str: &str) -> impl Iterator<Item = String> + '_ {
    args_str.split(' ').map(|s| s.to_string())
}

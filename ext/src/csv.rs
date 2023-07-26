pub use aio::*;

pub mod aio {
    use std::path::Path;
    use std::str::FromStr;

    use anyhow::{anyhow, Context};

    pub async fn load<T>(path: impl AsRef<Path>) -> anyhow::Result<Vec<T>>
    where
        T: Sized + FromStr<Err = anyhow::Error>,
    {
        let content = tokio::fs::read_to_string(&path).await.with_context(|| anyhow!("red file"))?;
        let lines = content.lines().skip(1);
        let outputs = lines
            .into_iter()
            .filter(|line| !line.is_empty())
            .map(T::from_str)
            .flatten()
            .collect();

        Ok(outputs)
    }
}

pub mod blocked {
    use std::path::Path;
    use std::str::Split;

    use anyhow::{anyhow, Context};

    pub fn load<F, T>(path: impl AsRef<Path>, fun: F) -> anyhow::Result<Vec<T>>
    where
        T: Sized,
        F: Fn(&mut Split<char>) -> anyhow::Result<T>,
    {
        let content = std::fs::read_to_string(&path).with_context(|| anyhow!("red file"))?;
        let lines = content.lines().skip(1);

        let lines = lines.into_iter().filter(|line| !line.is_empty());
        let outputs = lines
            .map(|line| {
                let mut fields = line.split(',');
                fun(&mut fields)
            })
            .flatten()
            .collect();
        Ok(outputs)
    }
}

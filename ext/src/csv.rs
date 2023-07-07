pub use aio::*;

pub mod aio {
    use std::path::Path;
    use std::str::Split;

    use anyhow::{anyhow, Context};

    pub async fn load<F, T>(path: impl AsRef<Path>, fun: F) -> anyhow::Result<Vec<T>>
    where
        T: Sized,
        F: Fn(&mut Split<char>) -> anyhow::Result<T>,
    {
        let content = tokio::fs::read_to_string(&path).await.with_context(|| anyhow!("red file"))?;
        let lines = content.lines().skip(1);
        let outputs = lines
            .into_iter()
            .filter(|line| !line.is_empty())
            .map(|line| {
                let mut fields = line.split(',');
                fun(&mut fields)
            })
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

#[cfg(test)]
mod tests {
    use anyhow::Context;

    #[tokio::test]
    async fn csv() {
        let path = std::env::var("HOME").unwrap() + "/.config/ticker/assets.csv";
        let holds = super::load(&path, |fields| {
            let day = fields.next().context("day")?;
            let symbol = fields.next().context("symbol")?;
            let name = fields.next().context("name")?;
            let cost = fields.next().context("cost")?.parse::<f32>()?;
            let volume = fields.next().context("volume")?;
            Ok((day.to_string(), symbol.to_string(), name.to_string(), cost, volume.to_string()))
        })
        .await;
        assert!(holds.is_ok());

        let holds = super::blocked::load(path, |fields| {
            let day = fields.next().context("day")?;
            let symbol = fields.next().context("symbol")?;
            let name = fields.next().context("name")?;
            let cost = fields.next().context("cost")?.parse::<f32>()?;
            let volume = fields.next().context("volume")?;
            Ok((day.to_string(), symbol.to_string(), name.to_string(), cost, volume.to_string()))
        });
        assert!(holds.is_ok());
    }
}

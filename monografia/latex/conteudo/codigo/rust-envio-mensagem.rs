    let response = self
        .net
        .get(ArcStr::from(&url), Some(headers))
        .await?;


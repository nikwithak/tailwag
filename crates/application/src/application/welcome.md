
        log::info!(
            r#"
=============================================
   Starting Web Application {}
=============================================
"#,
            &self.config.application_name,
        );
        log::debug!("CONFIGURED ENVIRONMENT: {:?}", &self);
        #[cfg(feature = "development")]
        {
            log::warn!(
                r#"
============================================================================="
    ++ {} IS RUNNING IN DEVELOPMENT MODE ++
    ++      DO NOT USE IN PRODUCTION     ++
============================================================================="
"#,
                self.config.application_name
            );
        }
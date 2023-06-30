use std::fmt;
use crate::puppet::writeln_opt;

use crate::puppet::*;

impl fmt::Display for PuppetAllowedUsers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PuppetAllowedUsers::OnlyAuthor => "only author",
                PuppetAllowedUsers::OnlyLicensee => "only licensee",
                PuppetAllowedUsers::Everyone => "Everyone",
            }
        )
    }
}

impl fmt::Display for PuppetAllowedRedistribution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PuppetAllowedRedistribution::Prohibited => "prohibited",
                PuppetAllowedRedistribution::ViralLicense => "viral license",
                PuppetAllowedRedistribution::CopyleftLicense => "copyleft license",
            }
        )
    }
}

impl fmt::Display for PuppetAllowedModification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PuppetAllowedModification::Prohibited => "prohibited",
                PuppetAllowedModification::AllowPersonal => "allow personal",
                PuppetAllowedModification::AllowRedistribute => "allow redistribute",
            }
        )
    }
}

impl fmt::Display for PuppetUsageRights {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "| allowed users:  {}", self.allowed_users)?;
        writeln!(f, "| violence:       {}", allowed_bool(self.allow_violence))?;
        writeln!(f, "| sexual:         {}", allowed_bool(self.allow_sexual))?;
        writeln!(
            f,
            "| commercial:     {}",
            allowed_bool(self.allow_commercial)
        )?;
        writeln!(f, "| redistribution: {}", self.allow_redistribution)?;
        writeln!(f, "| modification:   {}", self.allow_modification)?;
        writeln!(
            f,
            "| attribution: {}",
            if self.require_attribution {
                "required"
            } else {
                "not required"
            }
        )
    }
}

impl fmt::Display for PuppetMeta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.name {
            Some(ref name) => writeln_opt(f, "Name", &Some(name))?,
            None => {
                let no_name = "(No Name)";
                #[cfg(feature = "owo")]
                let no_name = {
                    use owo_colors::OwoColorize;
                    no_name.dimmed()
                };
                writeln!(f, "{no_name}")?
            }
        }

        writeln_opt(f, "Version", &Some(&self.version))?;
        writeln_opt(f, "Rigger", &self.rigger)?;
        writeln_opt(f, "Artist", &self.artist)?;

        if let Some(ref rights) = self.rights {
            writeln!(f, "Rights:")?;
            #[cfg(feature = "owo")]
            let rights = {
                use owo_colors::OwoColorize;
                rights.yellow()
            };
            writeln!(f, "{rights}")?;
        }

        writeln_opt(f, "Copyright", &self.copyright)?;
        writeln_opt(f, "License URL", &self.license_url)?;
        writeln_opt(f, "Contact", &self.contact)?;
        writeln_opt(f, "Reference", &self.reference)?;
        writeln_opt(f, "Thumbnail ID", &self.thumbnail_id)?;

        writeln_opt(
            f,
            "Preserve pixels",
            &Some(if self.preserve_pixels { "yes" } else { "no" }),
        )
    }
}

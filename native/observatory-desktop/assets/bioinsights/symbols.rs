// Original BioInsights embedded symbols and current webview button assets.
pub fn symbol(orange: bool, key: &str) -> Option<(&'static str, &'static [u8])> {
    match (orange, key) {
        (false, "PersonalCodexEntry") => Some((
            "theme-PersonalCodexEntry.png",
            include_bytes!("theme-PersonalCodexEntry.png"),
        )),
        (false, "Mark") => Some(("theme-Mark.png", include_bytes!("theme-Mark.png"))),
        (false, "Material") => Some(("theme-Material.png", include_bytes!("theme-Material.png"))),
        (false, "StarType") => Some(("theme-StarType.png", include_bytes!("theme-StarType.png"))),
        (false, "NotPredicted") => Some((
            "theme-NotPredicted.png",
            include_bytes!("theme-NotPredicted.png"),
        )),
        (false, "Tick") => Some(("theme-Tick.png", include_bytes!("theme-Tick.png"))),
        (false, "Likely") => Some(("theme-Likely.png", include_bytes!("theme-Likely.png"))),
        (false, "Maybe") => Some(("theme-Maybe.png", include_bytes!("theme-Maybe.png"))),
        (false, "Unlikely") => Some(("theme-Unlikely.png", include_bytes!("theme-Unlikely.png"))),
        (false, "PossibleCodexEntry") => Some((
            "theme-PossibleCodexEntry.png",
            include_bytes!("theme-PossibleCodexEntry.png"),
        )),
        (false, "PossibleCodexEntryNeverSeen") => Some((
            "theme-PossibleCodexEntryNeverSeen.png",
            include_bytes!("theme-PossibleCodexEntryNeverSeen.png"),
        )),
        (false, "ColourIsMissing") => Some((
            "theme-ColourIsMissing.png",
            include_bytes!("theme-ColourIsMissing.png"),
        )),
        (false, "Canonn") => Some(("theme-Canonn.png", include_bytes!("theme-Canonn.png"))),
        (false, "Horizons") => Some(("theme-Horizons.png", include_bytes!("theme-Horizons.png"))),
        (false, "CurrentBody") => Some((
            "theme-CurrentBody.png",
            include_bytes!("theme-CurrentBody.png"),
        )),
        (false, "OptionOn") => Some(("theme-OptionOn.png", include_bytes!("theme-OptionOn.png"))),
        (false, "OptionOff") => {
            Some(("theme-OptionOff.png", include_bytes!("theme-OptionOff.png")))
        }
        (false, "KoFi") => Some(("theme-KoFi.png", include_bytes!("theme-KoFi.png"))),
        (false, "Button-zoom") => Some((
            "theme-Button-zoom.png",
            include_bytes!("theme-Button-zoom.png"),
        )),
        (false, "Button-about") => Some((
            "theme-Button-about.png",
            include_bytes!("theme-Button-about.png"),
        )),
        (false, "Button-popout") => Some((
            "theme-Button-popout.png",
            include_bytes!("theme-Button-popout.png"),
        )),
        (false, "Button-main") => Some((
            "theme-Button-main.png",
            include_bytes!("theme-Button-main.png"),
        )),
        (false, "Button-unsold") => Some((
            "theme-Button-unsold.png",
            include_bytes!("theme-Button-unsold.png"),
        )),
        (false, "Button-settings") => Some((
            "theme-Button-settings.png",
            include_bytes!("theme-Button-settings.png"),
        )),
        (false, "Button-codexcompanion") => Some((
            "theme-Button-codexcompanion.png",
            include_bytes!("theme-Button-codexcompanion.png"),
        )),
        (true, "PersonalCodexEntry") => Some((
            "theme-orange-PersonalCodexEntry.png",
            include_bytes!("theme-orange-PersonalCodexEntry.png"),
        )),
        (true, "Mark") => Some((
            "theme-orange-Mark.png",
            include_bytes!("theme-orange-Mark.png"),
        )),
        (true, "Material") => Some((
            "theme-orange-Material.png",
            include_bytes!("theme-orange-Material.png"),
        )),
        (true, "StarType") => Some((
            "theme-orange-StarType.png",
            include_bytes!("theme-orange-StarType.png"),
        )),
        (true, "NotPredicted") => Some((
            "theme-orange-NotPredicted.png",
            include_bytes!("theme-orange-NotPredicted.png"),
        )),
        (true, "Tick") => Some((
            "theme-orange-Tick.png",
            include_bytes!("theme-orange-Tick.png"),
        )),
        (true, "Likely") => Some((
            "theme-orange-Likely.png",
            include_bytes!("theme-orange-Likely.png"),
        )),
        (true, "Maybe") => Some((
            "theme-orange-Maybe.png",
            include_bytes!("theme-orange-Maybe.png"),
        )),
        (true, "Unlikely") => Some((
            "theme-orange-Unlikely.png",
            include_bytes!("theme-orange-Unlikely.png"),
        )),
        (true, "PossibleCodexEntry") => Some((
            "theme-orange-PossibleCodexEntry.png",
            include_bytes!("theme-orange-PossibleCodexEntry.png"),
        )),
        (true, "PossibleCodexEntryNeverSeen") => Some((
            "theme-orange-PossibleCodexEntryNeverSeen.png",
            include_bytes!("theme-orange-PossibleCodexEntryNeverSeen.png"),
        )),
        (true, "ColourIsMissing") => Some((
            "theme-orange-ColourIsMissing.png",
            include_bytes!("theme-orange-ColourIsMissing.png"),
        )),
        (true, "Canonn") => Some((
            "theme-orange-Canonn.png",
            include_bytes!("theme-orange-Canonn.png"),
        )),
        (true, "Horizons") => Some((
            "theme-orange-Horizons.png",
            include_bytes!("theme-orange-Horizons.png"),
        )),
        (true, "CurrentBody") => Some((
            "theme-orange-CurrentBody.png",
            include_bytes!("theme-orange-CurrentBody.png"),
        )),
        (true, "OptionOn") => Some((
            "theme-orange-OptionOn.png",
            include_bytes!("theme-orange-OptionOn.png"),
        )),
        (true, "OptionOff") => Some((
            "theme-orange-OptionOff.png",
            include_bytes!("theme-orange-OptionOff.png"),
        )),
        (true, "KoFi") => Some((
            "theme-orange-KoFi.png",
            include_bytes!("theme-orange-KoFi.png"),
        )),
        (true, "Button-zoom") => Some((
            "theme-orange-Button-zoom.png",
            include_bytes!("theme-orange-Button-zoom.png"),
        )),
        (true, "Button-about") => Some((
            "theme-orange-Button-about.png",
            include_bytes!("theme-orange-Button-about.png"),
        )),
        (true, "Button-popout") => Some((
            "theme-orange-Button-popout.png",
            include_bytes!("theme-orange-Button-popout.png"),
        )),
        (true, "Button-main") => Some((
            "theme-orange-Button-main.png",
            include_bytes!("theme-orange-Button-main.png"),
        )),
        (true, "Button-unsold") => Some((
            "theme-orange-Button-unsold.png",
            include_bytes!("theme-orange-Button-unsold.png"),
        )),
        (true, "Button-settings") => Some((
            "theme-orange-Button-settings.png",
            include_bytes!("theme-orange-Button-settings.png"),
        )),
        (true, "Button-codexcompanion") => Some((
            "theme-orange-Button-codexcompanion.png",
            include_bytes!("theme-orange-Button-codexcompanion.png"),
        )),
        _ => None,
    }
}

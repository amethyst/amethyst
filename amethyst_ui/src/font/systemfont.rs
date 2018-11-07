use font_kit::{
    error::SelectionError,
    family_name::FamilyName,
    handle::Handle,
    properties::{Properties, Style},
    source::SystemSource,
};

/// Lists all installed font families on the system.
pub fn list_system_font_families() -> Result<Vec<String>, SelectionError> {
    let source = SystemSource::new();
    source.all_families()
}

/// Returns all the handles to the system fonts.
pub fn get_all_font_handles() -> Result<Vec<Handle>, SelectionError> {
    let source = SystemSource::new();
    let families = source.all_families()?;
    let family_handles = families
        .into_iter()
        .flat_map(|fam| source.select_family_by_name(&fam));
    let font_handles = family_handles
        .map(|fam| fam.fonts().to_owned())
        .flatten()
        .collect::<Vec<_>>();
    Ok(font_handles)
}

/// Returns the default system font.
pub fn default_system_font() -> Result<Handle, SelectionError> {
    let source = SystemSource::new();
    let default_fonts = &[
        FamilyName::Title("arial".to_string()),
        FamilyName::SansSerif,
        FamilyName::Monospace,
        FamilyName::Fantasy,
    ];
    source.select_best_match(default_fonts, Properties::new().style(Style::Normal))
}

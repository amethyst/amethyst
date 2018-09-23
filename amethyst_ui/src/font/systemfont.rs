use font_kit::handle::Handle;
use font_kit::error::SelectionError;
use font_kit::source::SystemSource;
use font_kit::family_name::FamilyName;
use font_kit::properties::{Properties, Style};

// Temporarily disabled because const can't contain heap allocations.
// Fantasy contains comic sans btw
//const DEFAULT_FONTS: &[FamilyName] = &[FamilyName::Title("arial".to_string()), FamilyName::SansSerif, FamilyName::Monospace, FamilyName::Fantasy];

/// Lists all installed font families on the system.
pub fn list_system_font_families() -> Result<Vec<String>, SelectionError> {
	let source = SystemSource::new();
    source.all_families()
}

/// Returns all the handles to the system fonts.
pub fn get_all_font_handles() -> Result<Vec<Handle>, SelectionError> {
	let source = SystemSource::new();
	let families = source.all_families()?;
	let family_handles = families.into_iter().flat_map(|fam| source.select_family_by_name(&fam));
	let font_handles = family_handles.map(|fam| fam.fonts().to_owned()).flatten().collect::<Vec<_>>();
    Ok(font_handles)
}

/// Returns the default system font.
pub fn default_system_font() -> Result<Handle, SelectionError> {
	let source = SystemSource::new();
	// TODO: remove once font_kit accepts titles with &str
	let default_fonts = &[FamilyName::Title("arial".to_string()), FamilyName::SansSerif, FamilyName::Monospace, FamilyName::Fantasy];
	source.select_best_match(default_fonts, Properties::new().style(Style::Normal))
}

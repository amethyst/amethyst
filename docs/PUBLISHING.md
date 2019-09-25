# Publishing Amethyst

Publishing a version of Amethyst requires incrementing all modified sub-crates 
and then publishing the sub-crates in the following order. You must comment out
dev-dependencies when publishing.

### Basic Crates

- `amethyst_config`
- `amethyst_derive`
- `amethyst_error`

### Core Crate

- `amethyst_core` <br/> dependencies:
  - Amethyst Error

### Third group

- `amethyst_assets` <br/> dependencies:
  - Amethyst Core
  - Amethyst Derive
  - Amethyst Error
- `amethyst_network` <br/> dependencies:
  - Amethyst Core
  - Amethyst Error
- `amethyst_window` <br/> dependencies:
  - Amethyst Core
  - Amethyst Config
  - Amethyst Error
  
### Fourth Group

- `amethyst_audio` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Core
  - Amethyst Error
- `amethyst_locale` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Core
  - Amethyst Error
- `amethyst_input` <br/> dependencies:
  - Amethyst Core
  - Amethyst Error
  - Amethyst Config
  - Amethyst Window
  
### Fifth Group

- `amethyst_controls` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Core
  - Amethyst Error
  - Amethyst Input
- `amethyst_rendy` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Config
  - Amethyst Core
  - Amethyst Derive
  - Amethyst Error
  - Amethyst Window
  
### Sixth Group

- `amethyst_tiles` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Rendy
  - Amethyst Core
  - Amethyst Error
  - Amethyst Window
- `amethyst_ui` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Audio
  - Amethyst Core
  - Amethyst Derive
  - Amethyst Error
  - Amethyst Input
  - Amethyst Rendy
  - Amethyst Window
- `amethyst_util` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Controls
  - Amethyst Core
  - Amethyst Error
  - Amethyst Derive
  - Amethyst Rendy
  - Amethyst Window
  
### Seventh Group

- `amethyst_animation` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Core
  - Amethyst Error
  - Amethyst Derive
  - Amethyst Rendy
  - Amethyst Ui
- `amethyst_gltf` <br/> dependencies:
  - Amethyst Assets
  - Amethyst Animation
  - Amethyst Core
  - Amethyst Error
  - Amethyst Rendy

### Amethyst!

Aka, `amethyst`

Dependencies:
- Amethyst Animation
- Amethyst Assets
- Amethyst Audio
- Amethyst Config
- Amethyst Core
- Amethyst Error
- Amethyst Controls
- Amethyst Derive
- Amethyst Gltf
- Amethyst Network
- Amethyst Locale
- Amethyst Rendy
- Amethyst Input
- Amethyst Ui
- Amethyst Utils
- Amethyst Window

### Post-Amethyst

- `amethyst_test` <br/> dependencies:
  - Amethyst


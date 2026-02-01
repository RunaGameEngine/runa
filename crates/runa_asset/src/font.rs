use crate::handle::Handle;

// Simple font asset that contains character textures
pub struct FontAsset {
    pub character_size: (u32, u32), // Size of each character in the font atlas
    pub characters: std::collections::HashMap<char, Handle<crate::texture::TextureAsset>>,
}

impl FontAsset {
    pub fn load_default() -> Self {
        // For now, we'll create a minimal font asset
        // In a real implementation, this would load a font file and generate character textures
        Self {
            character_size: (8, 8), // Default character size
            characters: std::collections::HashMap::new(),
        }
    }

    pub fn get_character_texture(&self, c: char) -> Option<&Handle<crate::texture::TextureAsset>> {
        self.characters.get(&c)
    }
}

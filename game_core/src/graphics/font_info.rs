use std::collections::HashMap;

#[derive(Clone)]
pub struct FontInfo {
    pub character_width: f32,
    pub character_height: f32,
    pub texture_layer: u32,
    pub characters: HashMap<char, (f32, f32)>,
    pub spacing: f32,
    pub line_height: f32,
}

impl FontInfo {
    pub fn new(texture_layer: u32, char_width: f32, char_height: f32) -> Self {
        let mut characters = HashMap::new();

        characters.insert('0', (char_width * 0.0, char_height * 3.0));
        characters.insert('1', (char_width * 1.0, char_height * 3.0));
        characters.insert('2', (char_width * 2.0, char_height * 3.0));
        characters.insert('3', (char_width * 3.0, char_height * 3.0));
        characters.insert('4', (char_width * 4.0, char_height * 3.0));
        characters.insert('5', (char_width * 5.0, char_height * 3.0));
        characters.insert('6', (char_width * 6.0, char_height * 3.0));
        characters.insert('7', (char_width * 7.0, char_height * 3.0));
        characters.insert('8', (char_width * 8.0, char_height * 3.0));
        characters.insert('9', (char_width * 9.0, char_height * 3.0));

        characters.insert('A', (char_width * 1.0, char_height * 4.0));
        characters.insert('B', (char_width * 2.0, char_height * 4.0));
        characters.insert('C', (char_width * 3.0, char_height * 4.0));
        characters.insert('D', (char_width * 4.0, char_height * 4.0));
        characters.insert('E', (char_width * 5.0, char_height * 4.0));
        characters.insert('F', (char_width * 6.0, char_height * 4.0));
        characters.insert('G', (char_width * 7.0, char_height * 4.0));
        characters.insert('H', (char_width * 8.0, char_height * 4.0));
        characters.insert('I', (char_width * 9.0, char_height * 4.0));
        characters.insert('J', (char_width * 10.0, char_height * 4.0));
        characters.insert('K', (char_width * 11.0, char_height * 4.0));
        characters.insert('L', (char_width * 12.0, char_height * 4.0));
        characters.insert('M', (char_width * 13.0, char_height * 4.0));
        characters.insert('N', (char_width * 14.0, char_height * 4.0));
        characters.insert('O', (char_width * 15.0, char_height * 4.0));
        characters.insert('P', (char_width * 0.0, char_height * 5.0));
        characters.insert('Q', (char_width * 1.0, char_height * 5.0));
        characters.insert('R', (char_width * 2.0, char_height * 5.0));
        characters.insert('S', (char_width * 3.0, char_height * 5.0));
        characters.insert('T', (char_width * 4.0, char_height * 5.0));
        characters.insert('U', (char_width * 5.0, char_height * 5.0));
        characters.insert('V', (char_width * 6.0, char_height * 5.0));
        characters.insert('W', (char_width * 7.0, char_height * 5.0));
        characters.insert('X', (char_width * 8.0, char_height * 5.0));
        characters.insert('Y', (char_width * 9.0, char_height * 5.0));
        characters.insert('Z', (char_width * 10.0, char_height * 5.0));

        characters.insert('a', (char_width * 1.0, char_height * 6.0));
        characters.insert('b', (char_width * 2.0, char_height * 6.0));
        characters.insert('c', (char_width * 3.0, char_height * 6.0));
        characters.insert('d', (char_width * 4.0, char_height * 6.0));
        characters.insert('e', (char_width * 5.0, char_height * 6.0));
        characters.insert('f', (char_width * 6.0, char_height * 6.0));
        characters.insert('g', (char_width * 7.0, char_height * 6.0));
        characters.insert('h', (char_width * 8.0, char_height * 6.0));
        characters.insert('i', (char_width * 9.0, char_height * 6.0));
        characters.insert('j', (char_width * 10.0, char_height * 6.0));
        characters.insert('k', (char_width * 11.0, char_height * 6.0));
        characters.insert('l', (char_width * 12.0, char_height * 6.0));
        characters.insert('m', (char_width * 13.0, char_height * 6.0));
        characters.insert('n', (char_width * 14.0, char_height * 6.0));
        characters.insert('o', (char_width * 15.0, char_height * 6.0));
        characters.insert('p', (char_width * 0.0, char_height * 7.0));
        characters.insert('q', (char_width * 1.0, char_height * 7.0));
        characters.insert('r', (char_width * 2.0, char_height * 7.0));
        characters.insert('s', (char_width * 3.0, char_height * 7.0));
        characters.insert('t', (char_width * 4.0, char_height * 7.0));
        characters.insert('u', (char_width * 5.0, char_height * 7.0));
        characters.insert('v', (char_width * 6.0, char_height * 7.0));
        characters.insert('w', (char_width * 7.0, char_height * 7.0));
        characters.insert('x', (char_width * 8.0, char_height * 7.0));
        characters.insert('y', (char_width * 9.0, char_height * 7.0));
        characters.insert('z', (char_width * 10.0, char_height * 7.0));

        characters.insert(' ', (char_width * 0.0, char_height * 2.0));

        characters.insert(':', (char_width * 10.0, char_height * 3.0));

        FontInfo {
            character_width: char_width,
            character_height: char_height,
            texture_layer,
            characters,
            spacing: char_width * 0.8,
            line_height: char_height * 1.2,
        }
    }

    pub fn get_character_position(&self, c: char) -> Option<(f32, f32)> {
        self.characters.get(&c).copied()
    }
}

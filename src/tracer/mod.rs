mod types;

pub mod tracer {
    use std::collections::HashMap;
    use std::fmt;
    use std::fs::File;
    use std::io::{BufRead, BufReader};

    use crate::tracer::types::{Color, Entity, Fov, Position, Screen, Surface};

    pub struct Model {
        background: Color,
        eyep: Position,
        lookp: Position,
        up: (u8, u8, u8),
        fov: Fov,
        screen: Screen,
        entities: Vec<Entity>,
    }

    impl Model {
        pub fn new(input_file_path: &str) -> Result<Self, ModelError> {
            let open_file_result = match File::open(input_file_path) {
                Ok(input_file) => input_file,
                Err(error) => {
                    return Err(ModelError::FailedToOpenInputFile(error.to_string()));
                }
            };

            let file_reader = BufReader::new(open_file_result);
            return match parse(file_reader) {
                Ok(model) => Ok(model),
                Err(error) => Err(error),
            };
        }
    }

    #[derive(Debug)]
    pub enum ModelError {
        FailedToOpenInputFile(String),
        ErrorParsingInputFile(String),
    }

    impl fmt::Display for ModelError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                ModelError::FailedToOpenInputFile(error_message) => {
                    write!(f, "Failed to open input file: {}", error_message)
                }
                ModelError::ErrorParsingInputFile(error_message) => {
                    write!(f, "Error parsing input file: {}", error_message)
                }
            }
        }
    }

    impl std::error::Error for ModelError {}

    pub fn render() {}
    pub fn write() {}

    fn parse(input_file_buf_reader: BufReader<File>) -> Result<Model, ModelError> {
        let mut background = Color { r: 0, g: 0, b: 0 };
        let mut eyep = Position { x: 0, y: 0, z: 0 };
        let mut lookp = Position { x: 0, y: 0, z: 0 };
        let mut up = (0u8, 0u8, 0u8);
        let mut fov = Fov { horz: 0, vert: 0 };
        let screen = Screen {
            width: 0,
            height: 0,
        };
        let mut entities: Vec<Entity> = Vec::new();
        let mut surfaces: HashMap<String, Surface> = HashMap::new();

        for (line_number, line_read_result) in input_file_buf_reader.lines().enumerate() {
            if line_read_result.is_err() {
                return Err(ModelError::ErrorParsingInputFile(format!(
                    "io error reading input file at line {}",
                    line_number
                )));
            }

            let line_iter = line_read_result.unwrap();
            let mut line_split_iter = line_iter.split(' ');
            let maybe_keyword = line_split_iter.next();

            // line has no content, got to next line
            if maybe_keyword.is_none() {
                continue;
            }

            let keyword = maybe_keyword.unwrap();

            println!("keyword is {}", keyword);
            let mut line_parts = String::new();
            let mut next_word = line_split_iter.next();
            while next_word.is_some() {
                line_parts.push_str(next_word.unwrap());

                next_word = line_split_iter.next();

                if next_word.is_some() {
                    line_parts.push_str(" | ")
                }
            }
            println!("rest of the line is {}", line_parts)
        }

        return Ok(Model {
            background,
            eyep,
            lookp,
            up,
            fov,
            screen,
            entities,
        });
    }
}

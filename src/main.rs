use std::{
    io::Read,
    time::{Duration, Instant},
};

type BFResult<T> = Result<T, BFError>;

#[derive(Debug)]
enum BFError {
    IoError(std::io::Error),
    ArgError,
    NegativeAddressError,
    ParenthesesPairingError,
    InvalidCharacter,
}

impl From<std::io::Error> for BFError {
    fn from(value: std::io::Error) -> Self {
        BFError::IoError(value)
    }
}

impl std::fmt::Display for BFError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BFError::IoError(err) => write!(f, "{}", err),
            BFError::ArgError => write!(f, "Špatný počet argumentů"),
            BFError::NegativeAddressError => write!(
                f,
                "Brainfuck program tried going into negative memory adresses."
            ),
            _ => write!(f, "Nechce se mi implementovat, mrdám to"),
        }
    }
}

fn get_file() -> Result<String, BFError> {
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() < 2 {
        return Err(BFError::ArgError);
    }

    std::fs::read_to_string(&arguments[1]).map_err(BFError::from)
}

fn check_char(char: &char) -> bool {
    let valid_chars: Vec<char> = vec!['>', '<', '+', '-', '.', ',', '[', ']'];
    for vchar in valid_chars.into_iter() {
        if char == &vchar {
            return true;
        }
    }
    false
}

struct Program {
    memory: Vec<u8>,
    pointer: i16,
    code: String,
    counter: i32,
    output_log: String,
}

impl Program {
    fn build(code: String) -> Self {
        let pure_code: String = code.chars().filter(check_char).collect();

        Self {
            memory: vec![0; 65535],
            pointer: 0,
            code: pure_code,
            counter: 0,
            output_log: String::new(),
        }
    }

    fn next_codelet(&self) -> char {
        if let Some((_, c)) = self
            .code
            .char_indices()
            .nth(self.counter.try_into().unwrap())
        {
            c
        } else {
            panic!("Index out of range.")
        }
    }

    fn step(&mut self) -> BFResult<bool> {
        let codelet: char = self.next_codelet();
        let current_memory: &mut u8 = &mut self.memory[self.pointer as usize];
        let mut cancel_step: bool = false;

        match codelet {
            '<' => {
                if (self.pointer - 1) == -1 {
                    return Err(BFError::NegativeAddressError);
                }
                self.pointer -= 1;
            }
            '>' => {
                self.pointer += 1;
            }
            '+' => {
                if (*current_memory as i32 + 1) > 255 {
                    *current_memory = 0;
                } else {
                    *current_memory += 1;
                }
            }
            '-' => {
                if (*current_memory as i32 - 1) == -1 {
                    *current_memory = 255;
                } else {
                    *current_memory -= 1;
                }
            }
            '.' => {
                let conversion_result: char =
                    char::from_u32(*current_memory as u32).unwrap_or('\0');
                if conversion_result == '\0' {
                    print!("{}", *current_memory);
                    self.output_log += format!("{}", *current_memory).as_str();
                } else {
                    print!("{}", conversion_result);
                    self.output_log.push(conversion_result)
                };
            }
            ',' => {
                let mut input = [0];
                std::io::stdin()
                    .read_exact(&mut input)
                    .expect("Failed to read user input");
                *current_memory = input[0];
            }
            '[' => {
                if *current_memory == 0 {
                    let mut tracker: i32 = 1;
                    let mut vcounter: i32 = self.counter + 1;

                    for tt in self.code.chars().skip(vcounter as usize) {
                        match tt {
                            '[' => {
                                tracker += 1;
                            }
                            ']' => {
                                tracker -= 1;
                            }
                            _ => {}
                        }
                        if tracker == 0 {
                            break;
                        }
                        vcounter += 1;
                    }

                    if tracker != 0 {
                        return Err(BFError::ParenthesesPairingError);
                    }

                    //println!("Closing ] found at {}", vcounter);
                    cancel_step = true;
                    self.counter = vcounter;
                }
            }
            ']' => {
                if *current_memory != 0 {
                    let mut tracker: i32 = 1;
                    let mut vcounter: i32 = self.counter - 1;

                    let instruction_vector: Vec<char> = self.code.chars().collect();
                    if instruction_vector.get(vcounter as usize).is_some() {
                        for tt in instruction_vector[..=vcounter as usize].iter().rev() {
                            //println!("Mathing {}, tracker is: {}", tt, tracker);

                            match tt {
                                ']' => {
                                    tracker += 1;
                                }
                                '[' => {
                                    tracker -= 1;
                                }
                                _ => {}
                            }
                            if tracker == 0 {
                                //println!("breaking");
                                break;
                            }

                            vcounter -= 1;
                        }
                    }

                    if tracker != 0 {
                        return Err(BFError::ParenthesesPairingError);
                    }

                    //println!("Closing [ found at {}", vcounter);
                    cancel_step = true;
                    self.counter = vcounter;
                }
            }
            _ => {
                return Err(BFError::InvalidCharacter);
            }
        }

        // Increase program counter
        if !cancel_step {
            self.counter += 1;
        }

        Ok(true)
    }

    fn run(&mut self) {
        let start_time: Instant = Instant::now();
        let codelet_count: i32 = self.code.chars().count().try_into().unwrap();
        println!("Running bf program. Instruction amount: {}", &codelet_count);
        println!("Pure code:");
        println!("{}", self.code);
        println!("---------------------------------------");
        // Main program loop
        while self.counter < codelet_count {
            self.step().unwrap_or_else(|err| {
                eprintln!(
                    "Brainfuck program halted at valid character no. {}, Reason: {}",
                    self.counter, err
                );
                std::process::exit(1);
            });
        }

        let elapsed: Duration = start_time.elapsed();
        println!("---------------------------------------");
        println!(
            "Execution succesful! Took {} microseconds",
            elapsed.as_micros()
        );
        println!("Program stopped at count {}.", self.counter);
        std::process::exit(0);
    }

    fn render_memory(&self) {
        println!("? = Invisible ascii character\n¿ = Empty cell (0)");

        let mut res: String = "Memory: ".to_owned();
        let mut point: String = "Point:  ".to_owned();
        for index in 0..100 {
            let cell: u8 = self.memory[index as usize];
            if cell == 0 {
                res.push('¿');
                continue;
            }

            if cell < 31 {
                res.push('?');
                continue;
            }

            let conversion_result: char = char::from_u32(cell as u32).unwrap_or('\0');
            if conversion_result == '\0' {
                res.push('?');
            } else {
                res.push(conversion_result);
            };
        }

        for _ in 0..self.pointer {
            point.push(' ')
        }
        point.push('↥');
        println!("{}", res);
        println!("{}", point);
        println!("Output so far: {}", self.output_log);
    }

    fn diagnostic_run(&mut self) {
        let codelet_count: i32 = self.code.chars().count().try_into().unwrap();
        while self.counter < codelet_count {
            self.step().unwrap_or_else(|err| {
                eprintln!(
                    "Brainfuck program halted at valid character no. {}, Reason: {}",
                    self.counter, err
                );
                std::process::exit(1);
            });
            clear_term();
            self.render_memory();
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn main_wrapper() -> BFResult<()> {
    let contents: String = get_file()?;

    let mut program: Program = Program::build(contents);

    // Run based on mode of operation
    let arguments: Vec<String> = std::env::args().collect();
    if arguments.len() > 2 {
        if arguments[2] == "visualised" {
            program.diagnostic_run();
        } else {
            program.run();
        }
    } else {
        program.run();
    }

    Ok(())
}

fn main() {
    if let Err(smthng) = main_wrapper() {
        eprintln!("Generický error handler:\n{}", smthng);
    }
}

fn clear_term() {
    print!("{}[2J", 27 as char);
}

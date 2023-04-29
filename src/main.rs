use std::env;
use std::fs;
use std::io;
use std::io::Read;
use std::process;
use std::thread;
use std::time;
use std::vec;

fn get_file() -> Result<String, &'static str> {
    let arguments: Vec<String> = env::args().collect();
    if arguments.len() < 2 {
        return Err("No arguments given!");
    }

    let file: Result<String, io::Error> = fs::read_to_string(&arguments[1]);

    if file.is_err() {
        return Err("Error while reading to string.");
    }

    Ok(file.unwrap())
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
    output_log: String
}

impl Program {
    fn build(code: String) -> Self {
        let pure_code: String = code.chars().filter(check_char).collect();

        Self {
            memory: vec![0; 65535],
            pointer: 0,
            code: pure_code,
            counter: 0,
            output_log: String::new()
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

    fn step(&mut self) -> Result<bool, &'static str> {
        let codelet: char = self.next_codelet();
        let current_memory: &mut u8 = &mut self.memory[self.pointer as usize];
        let mut cancel_step: bool = false;

        match codelet {
            '<' => {
                if (self.pointer - 1) == -1 {
                    return Err("Brainfuck program tried going into negative memory adresses.");
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
                    self.output_log += format!("{}",*current_memory).as_str();
                } else {
                    print!("{}", conversion_result);
                    self.output_log.push(conversion_result)
                };
            }
            ',' => {
                let mut input = [0];
                io::stdin()
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
                        return Err("1 A unclosed [] statement was found.");
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
                        return Err("2 A unclosed [] statement was found.");
                    }

                    //println!("Closing [ found at {}", vcounter);
                    cancel_step = true;
                    self.counter = vcounter;
                }
            }
            _ => {
                return Err("A invalid character got into the main loop.");
            }
        }

        // Increase program counter
        if !cancel_step {
            self.counter += 1;
        }

        Ok(true)
    }

    fn run(&mut self) {
        let start_time: time::Instant = time::Instant::now();
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
                process::exit(1);
            });
        }

        let elapsed: time::Duration = start_time.elapsed();
        println!("---------------------------------------");
        println!(
            "Execution succesful! Took {} microseconds",
            elapsed.as_micros()
        );
        println!("Program stopped at count {}.", self.counter);
        process::exit(0);
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
        println!("Output so far: {}",self.output_log);
    }

    fn diagnostic_run(&mut self) {
        let codelet_count: i32 = self.code.chars().count().try_into().unwrap();
        while self.counter < codelet_count {
            self.step().unwrap_or_else(|err| {
                eprintln!(
                    "Brainfuck program halted at valid character no. {}, Reason: {}",
                    self.counter, err
                );
                process::exit(1);
            });
            clear_term();
            self.render_memory();
            thread::sleep(time::Duration::from_millis(10));
        }
    }
}

fn main() {
    let contents: String = get_file().unwrap_or_else(|err: &str| {
        eprintln!("Couldn't read file, {}", err);
        process::exit(1);
    });

    let mut program: Program = Program::build(contents);

    //program.run();
    program.diagnostic_run();
}

fn clear_term() {
    print!("{}[2J", 27 as char);
}

use chrono::{Datelike, Local, NaiveDate, Weekday, Duration};
use std::env;
use sys_locale::get_locale;

const MONTHS_RU: [&str; 12] = [
    "Январь", "Февраль", "Март", "Апрель", "Май", "Июнь",
    "Июль", "Август", "Сентябрь", "Октябрь", "Ноябрь", "Декабрь",
];
const MONTHS_EN: [&str; 12] = [
    "January", "February", "March", "April", "May", "June",
    "July", "August", "September", "October", "November", "December",
];

const RED: &str = "\x1b[31m";
const INVERT: &str = "\x1b[7m";
const RESET: &str = "\x1b[0m";

fn print_help() {
    println!("Usage: dcal [options]");
    println!();
    println!("Options:");
    println!("  -h          Show this help message");
    println!("  -v          Show version information");
    println!("  -<1-12>     Display specified number of months starting from current");
    println!("  -c          Display 3 months, with the current month in the center");
    println!("  -g          Display the current year fully (4x3 grid)");
    println!("  -x<year>    Display the specified year fully (e.g., -x2021)");
    println!("  -b          Draw a beautiful border around each month (can be combined)");
    println!("  -e          Force English language output");
    println!("  -r          Force Russian language output");
    println!("  -m          Start the week on Sunday instead of Monday");
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    for arg in &args {
        if arg == "-h" || arg == "--help" {
            print_help();
            std::process::exit(0);
        } else if arg == "-v" || arg == "--version" {
            println!("dcal version 0.1.0");
            std::process::exit(0);
        }
    }

    let locale = get_locale().unwrap_or_else(|| "en".to_string());
    let mut is_ru = locale.starts_with("ru");
    let mut lang_overridden = false;

    let now = Local::now().date_naive();
    let current_year = now.year();
    let current_month = now.month() as i32;

    let mut start_year = current_year;
    let mut start_month = current_month;
    let mut months_count = 1;
    let mut use_border = false;
    let mut sunday_first = false;
    let mut cols_count = 4;
    let mut mode_selected = false;
    let mut show_weeks_total = false;
    let mut is_year_mode = false; // Флаг полного вывода года

    for arg in &args {
        if !arg.starts_with('-') || arg.len() < 2 {
            eprintln!("Error: Unknown argument {}", arg);
            std::process::exit(1);
        }

        if arg.starts_with("-x") {
            if let Ok(year) = arg[2..].parse::<i32>() {
                start_year = year;
                start_month = 1;
                months_count = 12;
                cols_count = 4;
                mode_selected = true;
                show_weeks_total = true;
                is_year_mode = true;
                continue;
            } else {
                eprintln!("Error: Invalid year format after -x");
                std::process::exit(1);
            }
        }

        let mut chars = arg.chars().skip(1).peekable();
        while let Some(c) = chars.next() {
            if c == 'b' {
                use_border = true;
            } else if c == 'm' {
                sunday_first = true;
            } else if c == 'e' {
                if !lang_overridden {
                    is_ru = false;
                    lang_overridden = true;
                }
            } else if c == 'r' {
                if !lang_overridden {
                    is_ru = true;
                    lang_overridden = true;
                }
            } else if c == 'g' {
                start_year = current_year;
                start_month = 1;
                months_count = 12;
                cols_count = 4;
                mode_selected = true;
                show_weeks_total = true;
                is_year_mode = true;
            } else if c == 'c' {
                let prev_month_date = NaiveDate::from_ymd_opt(current_year, current_month as u32, 1).unwrap() - Duration::days(1);
                start_year = prev_month_date.year();
                start_month = prev_month_date.month() as i32;
                months_count = 3;
                cols_count = 3;
                mode_selected = true;
            } else if c.is_ascii_digit() {
                let mut num_str = c.to_string();
                while let Some(&next_c) = chars.peek() {
                    if next_c.is_ascii_digit() {
                        num_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }
                if let Ok(count) = num_str.parse::<i32>() {
                    if (1..=12).contains(&count) {
                        months_count = count;
                        cols_count = if count < 4 { count as usize } else { 4 };
                        mode_selected = true;
                    } else {
                        eprintln!("Error: Number of months must be between 1 and 12");
                        std::process::exit(1);
                    }
                }
            } else {
                eprintln!("Error: Unknown flag -{}", c);
                std::process::exit(1);
            }
        }
    }

    if !mode_selected {
        cols_count = 1;
    }

    let mut months_to_render = Vec::new();
    let mut y = start_year;
    let mut m = start_month;

    for _ in 0..months_count {
        months_to_render.push((y, m));
        m += 1;
        if m > 12 {
            m = 1;
            y += 1;
        }
    }

    let chunks: Vec<&[(i32, i32)]> = months_to_render.chunks(cols_count).collect();
    for (i, chunk) in chunks.iter().enumerate() {
        print_months_row(chunk, is_ru, now, use_border, sunday_first, is_year_mode);
        if i < chunks.len() - 1 {
            println!();
        }
    }

    if show_weeks_total {
        if let Some(last_day_of_year) = NaiveDate::from_ymd_opt(start_year, 12, 28) {
            let total_weeks = last_day_of_year.iso_week().week();
            println!();
            if is_ru {
                println!("Всего недель в {} году: {}", start_year, total_weeks);
            } else {
                println!("Total weeks in year {}: {}", start_year, total_weeks);
            }
        }
    }
}

// Функция теперь принимает флаг force_max_height. Если false — высота подстраивается под реальные недели ряда.
fn generate_month_lines(year: i32, month: i32, is_ru: bool, today: NaiveDate, use_border: bool, sunday_first: bool, required_weeks: usize) -> Vec<String> {
    let mut lines = Vec::new();

    let month_name = if is_ru { MONTHS_RU[(month - 1) as usize] } else { MONTHS_EN[(month - 1) as usize] };
    let header_text = format!("{} {}", month_name, year);
    
    let wdays_text = if sunday_first {
        if is_ru {
            format!("{red}Вс{reset} Пн Вт Ср Чт Пт {red}Сб{reset}", red = RED, reset = RESET)
        } else {
            format!("{red}Su{reset} Mo Tu We Th Fr {red}Sa{reset}", red = RED, reset = RESET)
        }
    } else {
        if is_ru {
            format!("Пн Вт Ср Чт Пт {red}Сб Вс{reset}", red = RED, reset = RESET)
        } else {
            format!("Mo Tu We Th Fr {red}Sa Su{reset}", red = RED, reset = RESET)
        }
    };

    if use_border {
        lines.push("┌────────────────────┐".to_string());
        lines.push(format!("│{:^20}│", header_text));
        lines.push("├────────────────────┤".to_string());
        lines.push(format!("│{}│", wdays_text));
    } else {
        lines.push(format!("{:<20}", format!("    {}", header_text)));
        lines.push(wdays_text);
    }

    let first_day = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
    let weekday_offset = if sunday_first {
        first_day.weekday().num_days_from_sunday() as usize
    } else {
        first_day.weekday().num_days_from_monday() as usize
    };

    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap()
    };
    let total_days = next_month_date.signed_duration_since(first_day).num_days();

    let mut current_line = String::new();
    for _ in 0..weekday_offset {
        current_line.push_str("   ");
    }

    let mut day_idx = weekday_offset;
    for day in 1..=total_days {
        let date = NaiveDate::from_ymd_opt(year, month as u32, day as u32).unwrap();
        let wday = date.weekday();
        let is_today = date == today;
        let is_weekend = wday == Weekday::Sat || wday == Weekday::Sun;

        let mut day_str = format!("{:>2}", day);

        if is_today && is_weekend {
            day_str = format!("{}{}{}{}", INVERT, RED, day_str, RESET);
        } else if is_today {
            day_str = format!("{}{}{}", INVERT, day_str, RESET);
        } else if is_weekend {
            day_str = format!("{}{}{}", RED, day_str, RESET);
        }

        current_line.push_str(&day_str);
        
        day_idx += 1;
        if day_idx % 7 == 0 {
            if use_border {
                lines.push(format!("│{}│", current_line));
            } else {
                lines.push(current_line.clone());
            }
            current_line.clear();
        } else {
            current_line.push(' ');
        }
    }

    if !current_line.is_empty() {
        while strip_ansi_len(&current_line) < 20 {
            current_line.push(' ');
        }
        if use_border {
            lines.push(format!("│{}│", current_line));
        } else {
            lines.push(current_line);
        }
    }

    // Расчет высоты: шапка (2 без рамки, 4 с рамкой) + требуемое число строк с датами
    let target_grid_height = if use_border { 4 + required_weeks } else { 2 + required_weeks };
    
    while lines.len() < target_grid_height {
        let mut empty_line = String::new();
        while empty_line.len() < 20 {
            empty_line.push(' ');
        }
        if use_border {
            lines.push(format!("│{}│", empty_line));
        } else {
            lines.push(empty_line);
        }
    }

    if use_border {
        lines.push("└────────────────────┘".to_string());
    }

    lines
}

// Вспомогательная функция для определения, сколько строк с цифрами требует конкретный месяц
fn get_required_weeks_for_month(year: i32, month: i32, sunday_first: bool) -> usize {
    let first_day = NaiveDate::from_ymd_opt(year, month as u32, 1).unwrap();
    let weekday_offset = if sunday_first {
        first_day.weekday().num_days_from_sunday() as usize
    } else {
        first_day.weekday().num_days_from_monday() as usize
    };
    let next_month_date = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, (month + 1) as u32, 1).unwrap()
    };
    let total_days = next_month_date.signed_duration_since(first_day).num_days() as usize;
    
    // Округляем вверх деление суммы смещения и общего числа дней на 7
    (weekday_offset + total_days + 6) / 7
}

fn print_months_row(chunk: &[(i32, i32)], is_ru: bool, today: NaiveDate, use_border: bool, sunday_first: bool, is_year_mode: bool) {
    // Вычисляем максимальное число недель среди месяцев в текущем ряду
    let mut required_weeks = 4;
    if is_year_mode {
        required_weeks = 6; // Для полного года держим монолитные 6 строк
    } else {
        for &(year, month) in chunk {
            let weeks = get_required_weeks_for_month(year, month, sunday_first);
            if weeks > required_weeks {
                required_weeks = weeks;
            }
        }
    }

    let mut all_months_lines = Vec::new();
    for &(year, month) in chunk {
        all_months_lines.push(generate_month_lines(year, month, is_ru, today, use_border, sunday_first, required_weeks));
    }

    // Итоговое число строк вывода
    let mut total_output_rows = required_weeks + 2; // Без рамки: недели + 2 строки шапки
    if use_border {
        total_output_rows = required_weeks + 5; // С рамкой: недели + 4 строки шапки + 1 нижняя линия
    }
    let block_width = if use_border { 22 } else { 20 };

    for line_idx in 0..total_output_rows {
        let mut row_output = String::new();
        for month_lines in &all_months_lines {
            let raw_line = &month_lines[line_idx];
            let mut padded_line = raw_line.clone();
            
            let visible_len = strip_ansi_len(&raw_line);
            if visible_len < block_width {
                for _ in 0..(block_width - visible_len) {
                    padded_line.push(' ');
                }
            }

            row_output.push_str(&padded_line);
            row_output.push_str("    ");
        }
        println!("{}", row_output.trim_end());
    }
}

fn strip_ansi_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_ansi = false;
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\x1b' {
            in_ansi = true;
            i += 1;
            continue;
        }
        if in_ansi {
            if chars[i] == 'm' {
                in_ansi = false;
            }
            i += 1;
            continue;
        }
        len += 1;
        i += 1;
    }
    len
}


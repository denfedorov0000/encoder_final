use num_bigint::BigUint;
use num_traits::{One, Zero, Euclid, ToPrimitive};
use base64;
use rand::seq::SliceRandom;
use std::time::Instant;
//use std::panic::{self, UnwindSafe};

// Максимальное значение для чисел в списке (тесты)
const VAL_BASE: u16 = 300;
// Максимальное значение для заголовка
const HEADER_BASE: u32 = 1 << 32 - 1;
// Максимальное количество повторений одного числа
const REPEAT_MAX_COUNT: u16 = 60000;
// Максимальное значение числа в списке
const MAX_NUM: u16 = 65500;

// Вычисляет максимальное число в списке
fn max_num(numbers: &Vec<u16>) -> u16 {
    *numbers.iter().max().unwrap_or(&0)
}

// Вычисляет максимальное количество повторяющихся чисел в отсортированном списке
fn max_repeats_in_sorted(numbers: &Vec<u16>) -> u16 {
    let mut max_count = 1;
    let mut current_count = 1;
    let mut old_num = numbers[0];

    for num in numbers.iter().skip(1) {
        if *num == old_num {
            current_count += 1;
            if current_count > max_count {
                max_count = current_count;
            }
            if current_count > REPEAT_MAX_COUNT {
                panic!("Количество повторяющихся чисел превышает допустимое значение");
            }
        } else {
            current_count = 1;
        }
        old_num = *num;
    }
    max_count
}

// Возвращает базу для полинома кодирования
fn base(max_val: u16, max_repeats: u16) -> u32 {
    (max_val as u32 + 1) * (max_repeats as u32 + 1) + 1
}

fn pack(num: u16, count: u16, max_num: u16) -> u32 {
    (num as u32) + (count as u32) * (max_num as u32 + 1)
}

fn unpack(packed: u32, max_num: u16) -> (u16, u16) {
    let num = packed % (max_num as u32 + 1);
    let count = packed / (max_num as u32 + 1);
    (num as u16, count as u16)
}

// Сериализует список чисел в строку base64
fn serialize_list(numbers_: &[u16]) -> String {
    let mut numbers = numbers_.to_vec();
    numbers.sort();
    numbers.reverse();

    let max_num = max_num(&numbers);
    let max_repeats = max_repeats_in_sorted(&numbers);

    let mut polynomial = BigUint::zero();
    let mut powerbase: BigUint = BigUint::one();

    let mut count: u16 = 1;
    let mut old_num: u16 = 0;

    let packed_header = (max_num as u32) + ((max_repeats as u32) << 16);

    // Добавляем заголовок в полином
    polynomial += &powerbase * BigUint::from(packed_header);
    powerbase *= BigUint::from(HEADER_BASE);

    if !numbers.is_empty() {
        old_num = numbers[0];
    }

    let v_base = base(max_num, max_repeats);
    //println!("se v_base {}",v_base);

    for num in numbers.iter().skip(1) {
        if *num < 1 || *num > MAX_NUM {
            panic!("Число выходит за допустимый диапазон");
        }
        if count > REPEAT_MAX_COUNT {
            panic!("Количество повторяющихся чисел превышает допустимое значение");
        }

        if *num == old_num {
            count += 1;
        } else {
            //println!("{} {},",count,old_num);
            polynomial += &powerbase * BigUint::from( pack(old_num, count, max_num) );
            powerbase *= BigUint::from(v_base);

            count = 1;
        }
        old_num = *num;
    }

    if !numbers.is_empty() {
        //println!("{} {},",count,old_num);
        polynomial += &powerbase * BigUint::from( pack(old_num, count, max_num) );
    }
        
    //println!("{}",polynomial.to_string());

    let bytes = polynomial.to_bytes_be();
    base64::encode(&bytes)
}

// Десериализует строку base64 обратно в список чисел
fn deserialize_list(encoded: &String) -> Result<Vec<u16>, String> {
    //println!("___ {}",encoded);

    let bytes = base64::decode(encoded).map_err(|e| format!("Некорректное base64-представление: {}", e))?;
    let mut polynomial = BigUint::from_bytes_be(&bytes);

    println!("{}",polynomial.to_string());

    // Извлекаем заголовок
    let (s_quotient, s_remainder) = polynomial.div_rem_euclid(&BigUint::from(HEADER_BASE));
    let packed_header = s_remainder.to_u32().ok_or("Ошибка при извлечении заголовка")?;

    let max_num = (packed_header % (1 << 16)).to_u16().ok_or("Ошибка при извлечении max_num")?;
    let max_repeats = (packed_header >> 16).to_u16().ok_or("Ошибка при извлечении max_repeats")?;

    let mut result = Vec::new();

    polynomial = s_quotient;

    let v_base = base(max_num, max_repeats);
    //println!("de v_base {}",v_base);
    
    while polynomial > BigUint::zero() {
        let (quotient, remainder) = polynomial.div_rem_euclid(&BigUint::from(v_base));
        let packed_batch = remainder.to_u32().ok_or("Ошибка при извлечении числа из полинома")?;

        let (num, count) = unpack(packed_batch, max_num);

        //print!("de {} {},", count, num);

        if num < 1 || num > max_num {
            return Err("Некорректное значение коэффициента ".to_string()+&num.to_string());
        }

        for _ in 0..count {
            result.push(num);
        }
        //println!("");

        polynomial = quotient;
    }

    Ok(result)
}


// Тест: проверка простого списка
fn test_simple_case(_size: usize) -> (Vec<u16>, Vec<u16>, bool) {
    let input = vec![1, 1, 2, 2, 2, 3];
    let encoded = serialize_list(&input);
    let mut s_input = input.clone();
    s_input.sort();
    s_input.reverse();
    let decoded = deserialize_list(&encoded).unwrap();
    (input.clone(), decoded.clone(), decoded == s_input)
}

// Тест: проверка списка с однозначными числами
fn test_all_single_digits(size: usize) -> (Vec<u16>, Vec<u16>, bool) {
    let input = (1..=9).cycle().take(size).collect::<Vec<_>>();
    let encoded = serialize_list(&input);
    let mut s_input = input.clone();
    s_input.sort();
    s_input.reverse();
    let decoded = deserialize_list(&encoded).unwrap();
    (input.clone(), decoded.clone(), decoded == s_input)
}

// Тест: проверка списка с двузначными числами
fn test_all_two_digits(size: usize) -> (Vec<u16>, Vec<u16>, bool) {
    let input = (10..=99).cycle().take(size).collect::<Vec<_>>();
    let encoded = serialize_list(&input);
    let mut s_input = input.clone();
    s_input.sort();
    s_input.reverse();
    let decoded = deserialize_list(&encoded).unwrap();
    (input.clone(), decoded.clone(), decoded == s_input)
}

// Тест: проверка списка с трёхзначными числами
fn test_all_three_digits(size: usize) -> (Vec<u16>, Vec<u16>, bool) {
    let input = (100..=300).cycle().take(size).collect::<Vec<_>>();
    let encoded = serialize_list(&input);
    let mut s_input = input.clone();
    s_input.sort();
    s_input.reverse();
    let decoded = deserialize_list(&encoded).unwrap();
    (input.clone(), decoded.clone(), decoded == s_input)
}

// Тест: проверка списка, где каждое число повторяется 3 раза
fn test_three_repeats_each(_size: usize) -> (Vec<u16>, Vec<u16>, bool) {
    let input = (1..=VAL_BASE)
        .flat_map(|x| vec![x; 3])
        .collect::<Vec<_>>();

    let encoded = serialize_list(&input);
    let mut s_input = input.clone();
    s_input.sort();
    s_input.reverse();
    let decoded = deserialize_list(&encoded).unwrap();
    (input.clone(), decoded.clone(), decoded == s_input)
}

// Тест: проверка случайного списка чисел
fn test_random_lists(size: usize) -> (Vec<u16>, Vec<u16>, bool) {
    let mut rng = rand::thread_rng();
    let pool: Vec<u16> = (1..=VAL_BASE).collect();
    let input: Vec<u16> = (0..size).map(|_| *pool.choose(&mut rng).unwrap()).collect();

    let encoded = serialize_list(&input);
    let mut s_input = input.clone();
    s_input.sort();
    s_input.reverse();
    let decoded = deserialize_list(&encoded).unwrap();
    (input.clone(), decoded.clone(), decoded == s_input)
}

// Запускает тест, выводит результаты и статистику
fn run_test<F>(test_name: &str, input_size: usize, test_func: F) where F: FnOnce(usize) -> (Vec<u16>, Vec<u16>, bool) {
    let start = Instant::now();
    let (input, decoded, passed) = test_func(input_size);
    let duration = start.elapsed();

    let mut sorted_input = input.clone();
    sorted_input.sort();
    sorted_input.reverse();

    let input_str = format!("{:?}", input).replace(" ", "");
    let sorted_input_str = format!("{:?}", sorted_input).replace(" ", "");
    let decoded_str = format!("{:?}", decoded).replace(" ", "");

    let encoded = serialize_list(&input);
    let compression_ratio = if input.len() > 0 {
        (100.0 * (encoded.len() as f32 / input_str.len() as f32)) as f32
    } else {
        0.0
    };

    println!("Test '{}':", test_name);
    println!("Raw input: {}", input_str);
    println!("Input (sorted): {}", sorted_input_str);
    println!("Encoded: {}", encoded);
    println!("Decoded: {}", decoded_str);
    println!("Compression ratio: {:.2}%", compression_ratio);
    println!("Status: {}", if passed { "PASS" } else { "FAIL" });
    println!("Time: {:?}", duration);
    println!();
}

// Основная функция: запускает все тесты
fn main() {
    run_test("test_simple_case", 0,|_| test_simple_case(0));
    run_test("test_all_single_digits", 9,|_| test_all_single_digits(9));
    run_test("test_all_two_digits", 90,|_| test_all_two_digits(90));
    run_test("test_all_three_digits", 200, |_| test_all_three_digits(200));
    run_test("test_three_repeats_each", 300,  |_| test_three_repeats_each(300));
    run_test("test_random_lists 50", 50,|_| test_random_lists(50));
    run_test("test_random_lists 100", 100,|_| test_random_lists(100));
    run_test("test_random_lists 500", 500,|_| test_random_lists(500));
    run_test("test_random_lists 1000", 1000,|_| test_random_lists(1000));
}

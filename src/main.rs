// main.rs
// Тестовое задание Фёдоров Денис Г. email: denfedorov@mail.ru,denfedorov0000@yandex.ru

// Компактная сереализация/десереализация списка чисел в строку base64
// Идея алгоритма - превратить список чисел в длинное целое с подсчетом повторений
// одного числа (предварительно отсортировав входные значения) - порядок чисел не учитывается
// Для этого используем упаковку значений и вычисление специального полинома с применением
// длинной целочисленной арифметики, далее применяем преобразование в строку base64
// Десереализация реализована обратным алгоритмом
// При старте программа запускает набор рекомендованных тестов выводя результаты на экран
// release версия работает в десятки раз быстрее и в 10 раз меньше в обьеме.

use num_bigint::BigUint;
use num_traits::{One, Zero, Euclid};
use num_traits::ToPrimitive;
use base64;
use rand::seq::SliceRandom;
use std::time::Instant;
//use rand::prelude::IteratorRandom;

// Базовое значение для чисел в списке
const VAL_BASE: u32 = 300;
// Максимальное количество повторений одного числа
const REPEAT_MAX_COUNT: u32 = 1000;
// Общая база степеней полинома для кодирования (VAL_BASE + REPEAT_MAX_COUNT + 1)
const BASE: u32 = VAL_BASE + REPEAT_MAX_COUNT + 1;

// Сериализует список чисел в строку base64
fn serialize_list(numbers_: &[u32]) -> String {
    let mut numbers = numbers_.to_vec();
    numbers.sort();
    numbers.reverse();

    let mut polynomial = BigUint::zero();
    let mut powerbase: BigUint = BigUint::one();

    let mut count: u32 = 0;
    let mut old_num: u32 = 0;

    for num in numbers {
        if num < 1 || num > VAL_BASE {
            panic!("Число выходит за допустимый диапазон");
        }
        if count > REPEAT_MAX_COUNT {
            panic!("Количество повторяющихся чисел превышает допустимое значение");
        }
        count +=1;
        if count > 0 && num == old_num {
            count +=1;
            continue;
        }
        else
        {
            // Упаковываем число и количество повторений в одно значение
            let packed_num = num + (count) * (VAL_BASE+1);
            polynomial += powerbase.clone() * BigUint::from(packed_num);
            powerbase *= BigUint::from(BASE);

            count = 0;
            old_num = 0;
        }
    }

    // Переводим результат в байты и кодируем в base64
    let bytes = polynomial.to_bytes_be();
    base64::encode(&bytes)
}

// Десериализует строку base64 обратно в список чисел
fn deserialize_list(encoded: &str) -> Vec<u32> {
    let bytes = base64::decode(encoded).expect("Некорректное base64-представление");
    let mut polynomial = BigUint::from_bytes_be(&bytes);

    let base = BigUint::from(BASE);
    let mut result = Vec::new();

    while polynomial > BigUint::zero() {
        let (quotient, remainder) = polynomial.div_rem_euclid(&base);
        let packed_num = remainder.to_u32().unwrap();

        // Извлекаем исходное число и количество повторений
        let num = packed_num % (VAL_BASE+1);
        let count = packed_num / (VAL_BASE+1);

        if num < 1 || num > VAL_BASE {
            panic!("Некорректное значение коэффициента");
        }

        // Добавляем число в результат, учитывая количество повторений
        for _ in 0..count {
            result.push(num);
        }

        polynomial = quotient;
    }

    result
}

// Тест: проверка простого списка
fn test_simple_case(_size: usize) -> (Vec<u32>, Vec<u32>, bool) {
    let input = vec![1, 1, 2, 2, 2, 3];
    let encoded = serialize_list(&input);
    let decoded = deserialize_list(&encoded);
    (input.clone(), decoded.clone(), decoded == input)
}

// Тест: проверка списка с однозначными числами
fn test_all_single_digits(size: usize) -> (Vec<u32>, Vec<u32>, bool) {
    let input = (1..=9).cycle().take(size).collect::<Vec<_>>();
    let encoded = serialize_list(&input);
    let decoded = deserialize_list(&encoded);
    (input.clone(), decoded.clone(), decoded == input)
}

// Тест: проверка списка с двузначными числами
fn test_all_two_digits(size: usize) -> (Vec<u32>, Vec<u32>, bool) {
    let input = (10..=99).cycle().take(size).collect::<Vec<_>>();
    let encoded = serialize_list(&input);
    let decoded = deserialize_list(&encoded);
    (input.clone(), decoded.clone(), decoded == input)
}

// Тест: проверка списка с трёхзначными числами
fn test_all_three_digits(size: usize) -> (Vec<u32>, Vec<u32>, bool) {
    let input = (100..=300).cycle().take(size).collect::<Vec<_>>();
    let encoded = serialize_list(&input);
    let decoded = deserialize_list(&encoded);
    (input.clone(), decoded.clone(), decoded == input)
}

// Тест: проверка списка, где каждое число повторяется 3 раза
fn test_three_repeats_each(_size: usize) -> (Vec<u32>, Vec<u32>, bool) {

    let input = (1..=VAL_BASE)
        .flat_map(|x| vec![x; 3])
        .collect::<Vec<_>>();

    let encoded = std::panic::catch_unwind(|| serialize_list(&input))
        .unwrap_or_else(|_| "invalid".to_string());

    let decoded = std::panic::catch_unwind(|| deserialize_list(&encoded))
        .unwrap_or_default();

    (input.clone(), decoded.clone(), decoded == input)
}

// Тест: проверка случайного списка чисел
fn test_random_lists(size: usize) -> (Vec<u32>, Vec<u32>, bool) {
    println!("Input size: {}", size);
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

    let pool: Vec<u32> = (1..=VAL_BASE).collect();
    let mut input: Vec<u32> = Vec::<u32>::new();

    let range = 1..=size;
    range.for_each(|_|{ input.push(pool.choose(&mut rng).unwrap().to_u32().expect("REASON")); } );

    let encoded = serialize_list(&input);
    let decoded = deserialize_list(&encoded);
    (input.clone(), decoded.clone(), decoded == input)
}


// Запускает тест, выводит результаты и статистику
fn run_test<F>(test_name: &str, test_func: F) where F: FnOnce() -> (Vec<u32>, Vec<u32>, bool) {
    let start = Instant::now();
    let (input, decoded, _passed) = test_func();
    let duration = start.elapsed();

    let mut sorted_input = input.clone();
    sorted_input.sort();
    sorted_input.reverse();

    // Убираем пробелы в выводе массивов
    let input_str = format!("{:?}", input).replace(" ", "");
    let sorted_input_str = format!("{:?}", sorted_input).replace(" ", "");
    let decoded_str = format!("{:?}", decoded).replace(" ", "");

    let encoded = serialize_list(&input);
    let compression_ratio = if input.len() > 0 {
        (100.0 * (encoded.len() as f32  / input_str.len() as f32) ) as f32
    } else {
        0.0
    };

    println!("Test '{}' : ", test_name);
    println!("Raw input: {}", input_str);
    println!("Input (sorted): {}", sorted_input_str);
    println!("Encoded: {}", encoded);
    println!("Decoded: {}", decoded_str);
    println!("Compression ratio: {:.2}%", compression_ratio);
    println!("Status: {}", if decoded == sorted_input { "PASS" } else { "FAIL" });
    println!("Time: {:?}", duration);
    println!();
}

// Основная функция: запускает все тесты
fn main() {
    run_test("test_simple_case", || test_simple_case(0));
    run_test("test_all_single_digits", || test_all_single_digits(9));
    run_test("test_all_two_digits", || test_all_two_digits(90));
    run_test("test_all_three_digits", || test_all_three_digits(201));
    run_test("test_three_repeats_each", || test_three_repeats_each(300));
    run_test("test_random_lists 50 ", || test_random_lists(50));
    run_test("test_random_lists 100 ", || test_random_lists(100));
    run_test("test_random_lists 500 ", || test_random_lists(500));
    run_test("test_random_lists 1000 ", || test_random_lists(1000));
}

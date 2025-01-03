// Лабораторная работа №3
// Тема: исследование свойств имитационной модели.
//     3. Для имитационной модели сложной системы согласно ЛР2 решить следующие задачи:
//         3.1. проверить гипотезу о нормальности распределения откликов;
//              -> Сделать histogram'y, проверить её по какой-то метрике
//         3.2. вычислить точечные и интервальные оценки откликов ИМ в опыте из 10 прогонов при уровне значимости 0.05;
//              -> Вычислить оценку
//         3.3. оценить зависимость точности имитации от количества прогонов; ????
//         3.4. оценить чувствительность откликов к вариациям переменных ИМ;
//         3.5. построить зависимость изменения какого-либо отклика в модельном времени,
//         выдвинуть и проверить гипотезу об уменьшении времени прогона, исключая переходный период;
//         3.6. проверить гипотезу о возможности постановки опыта с непрерывным прогоном.
//     4. Отобразить результаты решения задач с помощью диаграмм.
//
// - Во время симуляции изменять переменную и наблюдать за непрерывным откликом?
//
//
//    Тема: постановка экспериментов с имитационной моделью
//Для имитационной модели сложной системы согласно ЛР2 решить следующие задачи:
// 1. построить зависимость отклика от варьирования параметра модели на 7+ уровнях,
// 2. выполнить линейную и нелинейную (любую) аппроксимацию, сделать вывод о наилучшем приближении,
//      учитывая погрешность имитации;
// 3. реализовать эксперимент по сравнению трёх альтернатив использования объекта моделирования;
// 4. поставить двухфакторный эксперимент (4+ уровня для каждого фактора) и отобразить поверхность отклика.
// 5. Отобразить результаты решения задач с помощью диаграмм.

// Каждому выбрать по отклику
// отклик должен стабилизироваться

mod app;
mod chart;
mod config;
pub mod egui_charts;
mod event;
mod experiment;
mod history;
mod results;
mod scenario;
mod simulation;
mod statistic;
pub mod tasks;

use std::fs;

use app::EguiApp;
pub use config::{EstimationConfig, SimulationConfig};
pub use event::Event;
pub use experiment::ExperimentConfig;
pub use history::Log;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
pub use results::Results;
use scenario::{ScenarioConfig, ScenarioParameter};
pub use simulation::{Simulation, SimulationTick};
pub use statistic::Stats;

fn asdfmain() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "SAIMOD",
        native_options,
        Box::new(|cc| Ok(Box::new(EguiApp::new(cc)))),
    )
    .unwrap();
}

fn main() -> anyhow::Result<()> {
    // env_logger::builder()
    //     .filter_level(log::LevelFilter::Info)
    //     .init();

    let config = {
        let raw_config = std::fs::read_to_string("config.toml").expect("Failed to read config");
        toml::from_str::<EstimationConfig>(&raw_config).expect("Failed to parse config")
    };

    let directories = vec![
        "stats/3_1",
        "stats/3_2",
        "stats/3_3",
        "stats/3_4",
        "stats/3_5",
        "stats/3_6",
        "stats/4_1",
        "stats/4_2/1",
        "stats/4_2/2",
        "stats/4_2/3",
        "stats/4_3",
    ];

    for dir_path in directories.into_iter() {
        if fs::metadata(dir_path).is_ok() {
            fs::remove_dir_all(dir_path)?;
        }
        fs::create_dir_all(dir_path)?;
    }

    let tasks = [
        task_3_1, task_3_2, task_3_3, task_3_4, task_3_5, task_3_6, task_4_1, task_4_2, task_4_3,
    ];
    tasks.into_par_iter().for_each(|task| task(&config));

    Ok(())
}

fn task_3_1(config: &EstimationConfig) {
    let mut total_results = Results::zeros();
    let mut total_logs = Log::empty();
    let mut results = Vec::<Results>::new();

    let tmp = (0..config.experiment.total)
        .into_par_iter()
        .map(|_| {
            let mut sim = Simulation::with_config(config.simulation.clone());
            sim.run()
        })
        .collect::<Vec<_>>();

    tmp.into_iter().for_each(|(run_result, run_log)| {
        total_results.add_mut(run_result.clone());
        total_logs.add_mut(run_log);

        results.push(run_result);
    });

    total_results.norm_mut(config.experiment.total);
    total_logs.norm_mut(config.experiment.total);

    chart::Histogram::from_y_data(
        "Среднее кол-во свободных работников",
        results.iter().map(|r| r.average_free_workers).collect(),
    )
    .save("stats/3_1/average_free_workers", &config.stats)
    .unwrap();

    chart::Histogram::from_y_data(
        "Кол-во обслуженных клиентов",
        results.iter().map(|r| r.dispatched_clients).collect(),
    )
    .save("stats/3_1/dispatched_clients", &config.stats)
    .unwrap();

    chart::Histogram::from_y_data(
        "Время ожидания работника",
        results.iter().map(|r| r.average_worker_waiting_time).collect(),
    )
    .save("stats/3_1/waiting_time", &config.stats)
    .unwrap();
}

// Интервальная оценка двух непрерывных и одного дискретного откликов
fn task_3_2(config: &EstimationConfig) {
    let total = 10;
    let mut results = vec![];

    for _ in 0..total {
        let mut sim = Simulation::with_config(config.simulation.clone());
        let (run_result, _run_log) = sim.run();
        results.push(run_result);
    }

    chart::Linear::from_data(
        "Свободные работники от времени симуляции",
        (0..total).map(|v| v as f32).collect(),
        results.iter().map(|r| r.average_free_workers).collect(),
    )
    .use_approximation(false)
    .set_config(&config.stats)
    .save("stats/3_2/free_workers")
    .unwrap();

    chart::Linear::from_data(
        "Кол-во обслуженных клиентов",
        (0..total).map(|v| v as f32).collect(),
        results.iter().map(|r| r.dispatched_clients).collect(),
    )
    .use_approximation(false)
    .set_config(&config.stats)
    .save("stats/3_2/dispatched_clients")
    .unwrap();

    chart::Linear::from_data(
        "Среднее время ожидания работника",
        (0..total).map(|v| v as f32).collect(),
        results.iter().map(|r| r.average_worker_waiting_time).collect(),
    )
    .use_approximation(false)
    .set_config(&config.stats)
    .save("stats/3_2/waiting_time")
    .unwrap();
}

fn task_3_3(config: &EstimationConfig) {
    let total = 100;
    let window_size = 2;
    let mut results = vec![];

    for _ in 0..(total + window_size) {
        let mut sim = Simulation::with_config(config.simulation.clone());
        let (run_result, _run_log) = sim.run();
        results.push(run_result);
    }

    let mut values = vec![];

    for i in window_size..(total + window_size) {
        let window = results[0..i].to_vec();
        values.push(window);
    }

    chart::Linear::from_data(
        "Свободные работники от времени симуляции",
        (0..total).map(|v| v as f32).collect(),
        values
            .iter()
            .map(|data| {
                let processed = data
                    .iter()
                    .map(|r| r.average_free_workers as f64)
                    .collect::<Vec<_>>();

                let stats = Stats::new(&processed, &config.stats);
                (2.0 * stats.std_dev * stats.t_stat) as f32 / (data.len() as f32).sqrt()
            })
            .collect(),
    )
    .save("stats/3_3/free_workers")
    .unwrap();

    chart::Linear::from_data(
        "Кол-во обслуженных клиентов over Time",
        (0..total).map(|v| v as f32).collect(),
        values
            .iter()
            .map(|data| {
                let processed = data
                    .iter()
                    .map(|r| r.dispatched_clients as f64)
                    .collect::<Vec<_>>();

                let stats = Stats::new(&processed, &config.stats);
                (2.0 * stats.std_dev * stats.t_stat) as f32 / (data.len() as f32).sqrt()
            })
            .collect(),
    )
    .save("stats/3_3/DispatchedClients")
    .unwrap();

    chart::Linear::from_data(
        "Waiting time",
        (0..total).map(|v| v as f32).collect(),
        values
            .iter()
            .map(|data| {
                let processed = data
                    .iter()
                    .map(|r| r.average_worker_waiting_time as f64)
                    .collect::<Vec<_>>();

                let stats = Stats::new(&processed, &config.stats);
                (2.0 * stats.std_dev * stats.t_stat) as f32 / (data.len() as f32).sqrt()
            })
            .collect(),
    )
    .save("stats/3_3/waiting_time")
    .unwrap();
}

// Change variable and see difference
fn task_3_4(config: &EstimationConfig) {
    let mut config = config.clone();
    config.scenario = Some(ScenarioConfig {
        parameters: vec![ScenarioParameter {
            kind: scenario::ParameterKind::Clients,
            values: 30..80,
            step: 1,
        }],
    });

    scenario::run(config, "3_4");
}

fn task_3_5(config: &EstimationConfig) {
    let mut config = config.clone();
    config.experiment.continous = false;
    config.experiment.total = 10_000;

    experiment::run(config, "stats/3_5/");
}

fn task_3_6(config: &EstimationConfig) {
    let mut config = config.clone();
    config.experiment.continous = true;
    config.experiment.total = 10_000;

    experiment::run(config, "stats/3_6/");
}

fn task_4_1(config: &EstimationConfig) {
    let mut config = config.clone();
    config.scenario = Some(ScenarioConfig {
        parameters: vec![ScenarioParameter {
            kind: scenario::ParameterKind::Production, // Production
            values: 3..15,
            step: 1,
        }],
    });

    scenario::run(config, "4_1");
}

fn task_4_2(config: &EstimationConfig) {
    let mut config = config.clone();
    config.experiment.continous = false;

    config.simulation.workers = 2;
    config.simulation.dancing_time = 1..4;
    let r1 = experiment::run(config.clone(), "stats/4_2/1");

    config.simulation.workers = 5;
    config.simulation.dancing_time = 2..8;
    let r2 = experiment::run(config.clone(), "stats/4_2/2");

    config.simulation.workers = 10;
    config.simulation.dancing_time = 4..12;
    let r3 = experiment::run(config.clone(), "stats/4_2/3");

    chart::Bar::from_y_data(
        "BusyTables",
        vec![
            r1.average_busy_tables,
            r2.average_busy_tables,
            r3.average_busy_tables,
        ],
    )
    .save("stats/4_2/BusyTables")
    .unwrap();

    chart::Bar::from_y_data(
        "FreeWorkers",
        vec![
            r1.average_free_workers,
            r2.average_free_workers,
            r3.average_free_workers,
        ],
    )
    .save("stats/4_2/FreeWorkers")
    .unwrap();

    chart::Bar::from_y_data(
        "WaitingTime",
        vec![
            r1.average_worker_waiting_time,
            r2.average_worker_waiting_time,
            r3.average_worker_waiting_time,
        ],
    )
    .save("stats/4_2/WaitingTime")
    .unwrap();

    chart::Bar::from_y_data(
        "DispatchedClients",
        vec![
            r1.dispatched_clients,
            r2.dispatched_clients,
            r3.dispatched_clients,
        ],
    )
    .save("stats/4_2/DispatchedClients")
    .unwrap();
}

fn task_4_3(config: &EstimationConfig) {
    let mut config = config.clone();
    config.scenario = Some(ScenarioConfig {
        parameters: vec![
            ScenarioParameter {
                kind: scenario::ParameterKind::Production,
                values: 2..6,
                step: 1,
            },
            ScenarioParameter {
                kind: scenario::ParameterKind::Dancing,
                values: 2..8,
                step: 1,
            },
        ],
    });

    scenario::run(config, "4_3");
}

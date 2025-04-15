use crate::util::types::{QuestionCorrelationData, QuestionDataDiffs, QuestionImpactData};
use serde::Serialize;
use std::{
    fs::{File, create_dir_all},
    io::{BufWriter, Write},
};

static OUT_DIR: &str = "out";

pub fn write_correlation_data(data: &Vec<QuestionCorrelationData>) {
    write_data(&format!("{}/correlation_data.json", OUT_DIR), data);
}

pub fn write_impact_data(data: &Vec<QuestionImpactData>) {
    write_data(&format!("{}/impact_data.json", OUT_DIR), data);
}

pub fn write_diffs_data(data: &Vec<QuestionDataDiffs>) {
    write_data(&format!("{}/diffs.json", OUT_DIR), data);
}

fn write_data<T: Serialize>(filename: &str, data: &Vec<T>) {
    create_dir_all(OUT_DIR).unwrap();
    let file: File = File::create(filename).unwrap();
    let mut writer: BufWriter<File> = BufWriter::new(file);
    serde_json::to_writer_pretty(&mut writer, data).unwrap();
    writer.flush().unwrap();
}

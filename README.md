# rust-SPPM

清华大学计算机图形学基础真实感渲染大作业的一个rust实现（初步完成）

## 已完成的功能

+ 基于随机渐进式光子映射（SPPM）的真实感渲染

+ 基本几何体、几何变换的渲染

+ obj模型的读取和渲染

+ 多线程支持

+ 景深相机

+ 多重采样抗锯齿

## 待完成的功能

+ 纹理映射

## Build

`cargo build --release`

## Build and Run

`cargo run --release <scene_file> <output_file>`

由于image库的限制，`output_file`最好为`jpg`或`png`格式。
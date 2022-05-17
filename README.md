# mandelbrot
Mandlebrot set plot implementation in Rust

This is based on the example from the O'Reilly "Crab" book, but has some updates to use Rayon instead of crossbeam and also adapted to support color.
The purpose of this repo is really a learning tool to become more fluent with Rust language, ecosystem and tooling. You can see what the code produces by looking at mandel.png. There is are both single and multithreaded versions and a powershell script for capturing the runtimes. Multithreaded runs at least 5x faster than single threaded on my 8 core machine.

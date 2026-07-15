use std::time::Duration;

use renderer_debug::{Tolerance, compare};
use wgpu_l3::renderer_debug::{Case, Harness};

fn main() {
    if let Err(error) = run(std::env::args().skip(1).collect()) {
        eprintln!("renderer_debug: {error}");
        std::process::exit(1);
    }
}

fn run(args: Vec<String>) -> Result<(), String> {
    match args.as_slice() {
        [command] if command == "list" => {
            for case in Case::ALL {
                println!("{}", case.name());
            }
            Ok(())
        }
        [command, case] if command == "reference" => {
            let case = parse_case(case)?;
            let mut harness = Harness::new(1.0)?;
            let environment = harness.environment();
            let (image, sample) = harness.render_legacy(case)?;
            println!(
                "case={} scale={} width={} height={} elapsed_us={} os={} architecture={} adapter={:?} backend={} device_type={}",
                case.name(),
                harness.scale_factor(),
                image.width(),
                image.height(),
                sample.elapsed().as_micros(),
                environment.os(),
                environment.architecture(),
                environment.adapter_name(),
                environment.backend(),
                environment.device_type(),
            );
            Ok(())
        }
        [command] if command == "reference-all" => {
            for scale in [1.0, 1.25, 1.5, 2.0] {
                let mut harness = Harness::new(scale)?;
                let environment = harness.environment();
                for case in Case::ALL {
                    let (image, sample) = harness.render_legacy(case)?;
                    let nontransparent = image
                        .pixels()
                        .iter()
                        .filter(|pixel| pixel[3] > 0.0)
                        .count();
                    if case.expects_visible_pixels() != (nontransparent > 0) {
                        return Err(format!(
                            "{} at scale {scale} violated its visibility expectation",
                            case.name()
                        ));
                    }
                    validate_case(case, &image)?;
                    println!(
                        "case={} scale={} width={} height={} nontransparent={} elapsed_us={} adapter={:?} backend={} device_type={}",
                        case.name(),
                        harness.scale_factor(),
                        image.width(),
                        image.height(),
                        nontransparent,
                        sample.elapsed().as_micros(),
                        environment.adapter_name(),
                        environment.backend(),
                        environment.device_type(),
                    );
                }
            }
            Ok(())
        }
        [command] if command == "oracle-all" => {
            for scale in [1.0, 1.25, 1.5, 2.0] {
                let mut harness = Harness::new(scale)?;
                let environment = harness.environment();
                for case in Case::ALL {
                    let (legacy, candidate, sample) = harness.render_pair(case)?;
                    validate_case(case, &legacy)?;
                    validate_case(case, &candidate)?;
                    let difference = compare(&legacy, &candidate, Tolerance::Exact)?;
                    println!(
                        "case={} scale={} differing_pixels={} maximum_channel_delta={} legacy_us={} candidate_us={} adapter={:?} backend={} device_type={}",
                        case.name(),
                        harness.scale_factor(),
                        difference.differing_pixels,
                        difference.maximum_channel_delta,
                        sample.legacy().elapsed().as_micros(),
                        sample.candidate().elapsed().as_micros(),
                        environment.adapter_name(),
                        environment.backend(),
                        environment.device_type(),
                    );
                }
            }
            Ok(())
        }
        [command, case, iterations] if command == "bench" => {
            let case = parse_case(case)?;
            let iterations = iterations
                .parse::<usize>()
                .map_err(|_| "iterations must be a positive integer".to_owned())?;
            if iterations == 0 {
                return Err("iterations must be a positive integer".to_owned());
            }
            let mut harness = Harness::new(1.0)?;
            let environment = harness.environment();
            let _ = harness.render_legacy(case)?;
            let mut samples = Vec::with_capacity(iterations);
            for _ in 0..iterations {
                let (_, sample) = harness.render_legacy(case)?;
                samples.push(sample.elapsed());
            }
            samples.sort_unstable();
            println!(
                "case={} warmup=1 samples={} p50_us={} p95_us={} max_us={} os={} architecture={} adapter={:?} backend={} device_type={}",
                case.name(),
                iterations,
                percentile(&samples, 50).as_micros(),
                percentile(&samples, 95).as_micros(),
                samples.last().copied().unwrap_or_default().as_micros(),
                environment.os(),
                environment.architecture(),
                environment.adapter_name(),
                environment.backend(),
                environment.device_type(),
            );
            Ok(())
        }
        _ => Err(
            "usage: renderer_debug list | reference <case> | reference-all | oracle-all | bench <case> <iterations>"
                .to_owned(),
        ),
    }
}

fn parse_case(value: &str) -> Result<Case, String> {
    Case::from_name(value).ok_or_else(|| format!("unknown case: {value}"))
}

fn validate_case(case: Case, image: &wgpu_l3::renderer_debug::Image) -> Result<(), String> {
    if case != Case::TransparentPopup {
        return Ok(());
    }
    let x = image.width() as usize / 2;
    let y = image.height() as usize / 2;
    let sample = image.pixels()[y * image.width() as usize + x];
    let tolerance = 2.0 / 255.0;
    let expected_rgb = 64.0 / 255.0;
    let expected_alpha = 128.0 / 255.0;
    if (sample[3] - expected_alpha).abs() > tolerance
        || sample[0..3]
            .iter()
            .any(|channel| (*channel - expected_rgb).abs() > tolerance)
        || sample[0..3]
            .iter()
            .any(|channel| *channel > sample[3] + tolerance)
    {
        return Err(format!(
            "transparent popup must be sRGB-encoded then premultiplied; expected about [{expected_rgb}, {expected_rgb}, {expected_rgb}, {expected_alpha}], got {sample:?}"
        ));
    }
    Ok(())
}

fn percentile(samples: &[Duration], percentile: usize) -> Duration {
    let index = samples
        .len()
        .saturating_mul(percentile)
        .div_ceil(100)
        .saturating_sub(1)
        .min(samples.len().saturating_sub(1));
    samples.get(index).copied().unwrap_or_default()
}

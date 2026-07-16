use std::time::Duration;

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
        [command, case, scale] if command == "readback" => {
            let case = parse_case(case)?;
            let scale = scale
                .parse::<f32>()
                .map_err(|_| "scale must be a positive number".to_owned())?;
            let mut harness = harness(scale)?;
            let environment = harness.environment();
            let (image, sample) = harness.render(case)?;
            validate_case(case, &image)?;
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
            println!(
                "case={} scale={} width={} height={} nontransparent={} elapsed_us={} os={} architecture={} adapter={:?} backend={} device_type={}",
                case.name(),
                harness.scale_factor(),
                image.width(),
                image.height(),
                nontransparent,
                sample.elapsed().as_micros(),
                environment.os(),
                environment.architecture(),
                environment.adapter_name(),
                environment.backend(),
                environment.device_type(),
            );
            Ok(())
        }
        [command] if command == "readback-all" => {
            for scale in [1.0, 1.25, 1.5, 2.0] {
                let mut harness = harness(scale)?;
                let environment = harness.environment();
                for case in Case::ALL {
                    let (image, sample) = harness.render(case)?;
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
        [command, case] if command == "work" => {
            let case = parse_case(case)?;
            let mut harness = harness(1.0)?;
            print_work(case.name(), harness.work_receipt(case)?);
            Ok(())
        }
        [command, case] if command == "retention" => {
            let case = parse_case(case)?;
            let mut harness = harness(1.0)?;
            let receipt = harness.retention_receipt(case)?;
            print_work("first", receipt.first());
            print_work("unchanged", receipt.unchanged());
            print_work("recreated", receipt.recreated());
            print_work("retired", receipt.retired());
            Ok(())
        }
        [command] if command == "partial-update" => {
            let mut harness = harness(1.0)?;
            let receipt = harness.partial_update_receipt()?;
            print_work("first", receipt.first());
            print_work("changed", receipt.changed());
            print_work("surviving", receipt.surviving());
            print_work("retired", receipt.retired());
            Ok(())
        }
        [command, iterations] if command == "churn" => {
            let iterations = iterations
                .parse::<usize>()
                .map_err(|_| "iterations must be an integer of at least three".to_owned())?;
            let mut harness = harness(1.0)?;
            let receipt = harness.churn_receipt(iterations)?;
            println!(
                "iterations={} peak_resources={} peak_bytes={} post_warm_resource_range={:?} post_warm_byte_range={:?}",
                receipt.iterations(),
                receipt.peak_resources(),
                receipt.peak_bytes(),
                receipt.post_warm_resource_range(),
                receipt.post_warm_byte_range(),
            );
            print_work("settled", receipt.settled());
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
            let mut harness = harness(1.0)?;
            let environment = harness.environment();
            let _ = harness.render(case)?;
            let mut samples = Vec::with_capacity(iterations);
            for _ in 0..iterations {
                let (_, sample) = harness.render(case)?;
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
        [command] if command == "scroll-bench-list" => {
            for workload in wgpu_l3::diagnostics::ScrollBenchWorkload::ALL {
                println!("{}", workload.name());
            }
            Ok(())
        }
        [command, workload] if command == "scroll-bench" => run_scroll_bench(
            workload,
            wgpu_l3::diagnostics::OFFICIAL_PROPERTY_WARMUP,
            wgpu_l3::diagnostics::OFFICIAL_PROPERTY_SAMPLES,
        ),
        [command, workload, warmup, samples] if command == "scroll-bench" => {
            let warmup = warmup
                .parse::<usize>()
                .map_err(|_| "scroll-bench warmup must be a non-negative integer".to_owned())?;
            let samples = samples
                .parse::<usize>()
                .map_err(|_| "scroll-bench samples must be a positive integer".to_owned())?;
            run_scroll_bench(workload, warmup, samples)
        }
        [command] if command == "table-scroll-work" => run_table_scroll_work(1.0),
        [command, scale] if command == "table-scroll-work" => {
            let scale = scale
                .parse::<f32>()
                .map_err(|_| "scale must be a positive number".to_owned())?;
            if !scale.is_finite() || scale <= 0.0 {
                return Err("scale must be a positive number".to_owned());
            }
            run_table_scroll_work(scale)
        }
        [command] if command == "group-scroll-oracle" => {
            for scale in [1.0, 1.25, 1.5, 1.75, 2.0] {
                run_group_scroll_oracle(scale)?;
            }
            Ok(())
        }
        [command, scale] if command == "group-scroll-oracle" => {
            let scale = scale
                .parse::<f32>()
                .map_err(|_| "scale must be a positive number".to_owned())?;
            if !scale.is_finite() || scale <= 0.0 {
                return Err("scale must be a positive number".to_owned());
            }
            run_group_scroll_oracle(scale)
        }
        [command] if command == "tier-a-scroll-oracle" => {
            for scale in [1.0, 1.25, 1.5, 1.75, 2.0] {
                run_tier_a_scroll_oracle(scale)?;
            }
            Ok(())
        }
        [command, scale] if command == "tier-a-scroll-oracle" => {
            let scale = scale
                .parse::<f32>()
                .map_err(|_| "scale must be a positive number".to_owned())?;
            if !scale.is_finite() || scale <= 0.0 {
                return Err("scale must be a positive number".to_owned());
            }
            run_tier_a_scroll_oracle(scale)
        }
        [command] if command == "tier-a-negative-controls" => {
            pollster::block_on(
                wgpu_l3::diagnostics::require_payload_neutral_scroll_negative_controls(),
            )?;
            println!("oracle=tier-a-negative-controls executions=10 result=pass");
            Ok(())
        }
        _ => Err(
            "usage: renderer_debug list | readback <case> <scale> | readback-all | work <case> | retention <case> | partial-update | churn <iterations> | bench <case> <iterations> | scroll-bench-list | scroll-bench <workload> [warmup samples] | table-scroll-work [scale] | group-scroll-oracle [scale] | tier-a-scroll-oracle [scale] | tier-a-negative-controls"
                .to_owned(),
        ),
    }
}

fn run_table_scroll_work(scale: f32) -> Result<(), String> {
    let work = pollster::block_on(
        wgpu_l3::diagnostics::measure_control_gallery_horizontal_table_scroll(scale),
    )?;
    println!("workload=control-gallery-horizontal-table-scroll scale={scale}");
    print_work("property-hit", work);
    Ok(())
}

fn run_group_scroll_oracle(scale: f32) -> Result<(), String> {
    pollster::block_on(wgpu_l3::diagnostics::compare_group_under_scroll_first_tick(
        scale,
    ))?;
    println!("oracle=group-under-scroll-first-tick scale={scale} result=pass");
    Ok(())
}

fn run_tier_a_scroll_oracle(scale: f32) -> Result<(), String> {
    pollster::block_on(wgpu_l3::diagnostics::compare_payload_neutral_scroll_oracles(scale))?;
    println!("oracle=tier-a-payload-neutral-first-tick scale={scale} cases=8 result=pass");
    Ok(())
}

fn run_scroll_bench(workload: &str, warmup: usize, samples: usize) -> Result<(), String> {
    let workload = wgpu_l3::diagnostics::ScrollBenchWorkload::from_name(workload)
        .ok_or_else(|| format!("unknown scroll-bench workload: {workload}"))?;
    let receipt = wgpu_l3::diagnostics::run_scroll_bench(workload, warmup, samples)?;
    println!("{}", receipt.receipt_text(&git_commit()));
    Ok(())
}

fn git_commit() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|commit| commit.trim().to_owned())
        .filter(|commit| !commit.is_empty())
        .unwrap_or_else(|| "unknown".to_owned())
}

fn print_work(stage: &str, work: wgpu_l3::renderer_debug::Work) {
    println!(
        "stage={stage} node_rebuilds={} primitive_prepare_calls={} text_prepare_calls={} text_shape_calls={} content_upload_bytes={} property_upload_bytes={} viewport_property_upload_bytes={} node_property_upload_bytes={} scroll_property_upload_bytes={} text_property_upload_bytes={} unattributed_property_upload_bytes={} gpu_resources={} gpu_bytes={} gpu_creations={} gpu_replacements={} gpu_removals={} plan_rebuilds={} plan_reuses={} direct_surface_plans={} surface_sampling_plans={} draw_calls={} draw_passes={} explicit_copy_commands={} resource_transition_boundaries={} opaque_nodes={} blended_nodes={} opacity_unclassified_nodes={} effect_intermediate_clears={} effect_intermediate_clear_bytes={} effect_intermediate_composites={} effect_intermediate_composite_bytes={} largest_effect_intermediate_bytes={} target_bytes={}",
        work.scene_node_realization_rebuilds(),
        work.primitive_prepare_calls(),
        work.text_prepare_calls(),
        work.text_shape_calls(),
        work.content_upload_bytes(),
        work.property_upload_bytes(),
        work.viewport_property_upload_bytes(),
        work.node_property_upload_bytes(),
        work.scroll_property_upload_bytes(),
        work.text_property_upload_bytes(),
        work.unattributed_property_upload_bytes(),
        work.gpu_resource_count(),
        work.gpu_resource_bytes(),
        work.gpu_resource_creations(),
        work.gpu_resource_replacements(),
        work.gpu_resource_removals(),
        work.render_plan_rebuilds(),
        work.render_plan_reuses(),
        work.direct_surface_plans(),
        work.surface_sampling_plans(),
        work.draw_calls(),
        work.draw_passes(),
        work.explicit_copy_commands(),
        work.resource_transition_boundaries(),
        work.opaque_nodes(),
        work.blended_nodes(),
        work.opacity_unclassified_nodes(),
        work.effect_intermediate_clears(),
        work.effect_intermediate_clear_bytes(),
        work.effect_intermediate_composites(),
        work.effect_intermediate_composite_bytes(),
        work.largest_effect_intermediate_bytes(),
        work.target_bytes(),
    );
}

fn harness(scale_factor: f32) -> Result<Harness, String> {
    pollster::block_on(Harness::new(scale_factor))
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

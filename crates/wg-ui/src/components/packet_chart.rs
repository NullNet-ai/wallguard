use leptos::prelude::*;

/// SVG width and height used for all charts.
const SVG_W: f64 = 400.0;
const SVG_H: f64 = 100.0;

/// Padding inside the SVG so the polyline does not clip the axes.
const PAD_X: f64 = 30.0;
const PAD_Y: f64 = 10.0;

/// Convert a slice of (timestamp_s, packets_per_s) samples into an SVG
/// `<polyline points="…">` attribute value.  Returns an empty string when
/// there are fewer than two samples.
fn build_points(data: &[(f64, f64)]) -> String {
    if data.len() < 2 {
        return String::new();
    }

    let t_min   = data.first().map(|p| p.0).unwrap_or(0.0);
    let t_max   = data.last().map(|p| p.0).unwrap_or(1.0);
    let t_range = (t_max - t_min).max(1e-9);
    let v_max   = data.iter().map(|p| p.1).fold(0.0_f64, f64::max).max(1e-9);

    let plot_w = SVG_W - PAD_X;
    let plot_h = SVG_H - PAD_Y * 2.0;

    data.iter()
        .map(|(t, v)| {
            let x = PAD_X + (t - t_min) / t_range * plot_w;
            // Y = 0 at bottom (SVG_H − PAD_Y), v_max at top (PAD_Y)
            let y = (SVG_H - PAD_Y) - (v / v_max) * plot_h;
            format!("{:.1},{:.1}", x, y)
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[component]
pub fn PacketChart(
    #[prop(optional)] data: Option<Vec<(f64, f64)>>,
) -> impl IntoView {
    let data     = data.unwrap_or_default();
    let points   = build_points(&data);
    let has_data = !points.is_empty();

    let y_max_label = if has_data {
        let v_max = data.iter().map(|p| p.1).fold(0.0_f64, f64::max);
        format!("{:.0}", v_max)
    } else {
        "0".to_string()
    };

    // Pre-compute string attributes so nothing borrows `data` inside `view!`.
    let y_axis_x  = PAD_X.to_string();
    let y_axis_y1 = PAD_Y.to_string();
    let y_axis_y2 = (SVG_H - PAD_Y).to_string();
    let x_axis_x2 = SVG_W.to_string();
    let x_axis_y  = (SVG_H - PAD_Y).to_string();
    let lbl_x     = (PAD_X - 2.0).to_string();
    let lbl_top_y = (PAD_Y + 4.0).to_string();
    let lbl_bot_y = (SVG_H - PAD_Y).to_string();
    let vb        = format!("0 0 {} {}", SVG_W, SVG_H);
    let w_str     = SVG_W.to_string();
    let h_str     = SVG_H.to_string();

    view! {
        <svg
            class="packet-chart"
            attr:viewBox=vb
            attr:width=w_str
            attr:height=h_str
            attr:xmlns="http://www.w3.org/2000/svg"
        >
            // Y-axis line
            <line
                attr:x1=y_axis_x.clone()
                attr:y1=y_axis_y1
                attr:x2=y_axis_x.clone()
                attr:y2=y_axis_y2.clone()
                attr:stroke="#94a3b8"
                attr:stroke-width="1"
            />
            // X-axis line
            <line
                attr:x1=y_axis_x
                attr:y1=x_axis_y.clone()
                attr:x2=x_axis_x2
                attr:y2=x_axis_y
                attr:stroke="#94a3b8"
                attr:stroke-width="1"
            />
            // Y max label
            <text
                attr:x=lbl_x.clone()
                attr:y=lbl_top_y
                attr:text-anchor="end"
                attr:font-size="9"
                attr:fill="#64748b"
            >
                {y_max_label}
            </text>
            // Y zero label
            <text
                attr:x=lbl_x
                attr:y=lbl_bot_y
                attr:text-anchor="end"
                attr:font-size="9"
                attr:fill="#64748b"
            >
                "0"
            </text>

            <Show
                when=move || has_data
                fallback=|| view! {
                    <text
                        attr:x="215"
                        attr:y="55"
                        attr:text-anchor="middle"
                        attr:font-size="11"
                        attr:fill="#94a3b8"
                    >
                        "No data"
                    </text>
                }
            >
                <polyline
                    attr:points=points.clone()
                    attr:fill="none"
                    attr:stroke="#2563eb"
                    attr:stroke-width="1.5"
                    attr:stroke-linejoin="round"
                    attr:stroke-linecap="round"
                />
            </Show>
        </svg>
    }
}

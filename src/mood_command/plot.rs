use gnuplot::{AxesCommon,Auto,CloseSentinel,Figure,Format,GnuplotInitError};

const DATE_FORMAT: &str = "%d/%m/%Y"; // TODO: make it configurable

pub fn draw<Tx, Ty>(data: &[(Tx, Ty)]) -> Result<CloseSentinel, GnuplotInitError>
where
    Tx: gnuplot::DataType + Copy,
    Ty: gnuplot::DataType + Copy,
{
    let x: Vec<Tx> = data.iter().map(|v| v.0).collect();
    let y: Vec<Ty> = data.iter().map(|v| v.1).collect();

    let mut fg = Figure::new();
    fg.axes2d()
        .set_title("30-days moving cumulative mood", &[])
        .lines(x, y, &[])
        .set_x_ticks(Some((Auto, 0)), &[Format(DATE_FORMAT)], &[])
        .set_x_time(true);
    fg.show()
}

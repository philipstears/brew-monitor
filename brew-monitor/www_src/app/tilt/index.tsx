import * as React from "react";
import Chart from "react-apexcharts";

function useQuery(): URLSearchParams {
  return new URLSearchParams(window.location.search);
}

export interface TiltProps {
    color: string;
}

export interface TiltState {
    gravity_data: ChartDatum[],
    temperature_data: ChartDatum[],
    from: string | null,
    to: string | null;
}

type ChartDatum = [number, number];

export interface Reading {
    at: string,
    fahrenheit: number,
    gravity: number,
}

export class Tilt extends React.Component<TiltProps, TiltState> {
    constructor(props: TiltProps) {
        let query = useQuery();

        super(props);
        this.state = {
            gravity_data: [],
            temperature_data: [],
            from: query.get("from"),
            to: query.get("to"),
        };
        this.refreshTiltData();
    }

    render() {
        let graph = {
            options: {
                chart: {
                    id: "basic-bar",
                    background: "#000",
                },
                theme: {
                    mode: "dark",
                },
                stroke: {
                    curve: 'straight',
                    width: 1,
                },
                xaxis: {
                    type: 'datetime'
                },
                yaxis: [
                    {
                        name: "Gravity",
                        min: 1000,
                        max: (v: number) => {
                            // Round up to the nearest 10 gravity points
                            return Math.round((v + 5) / 10) * 10;
                        },
                    },
                    {
                        name: "Temperature",
                        opposite: true,
                    }
                ],
                annotations: {
                    yaxis: [
                        {
                            y: 1007,
                            borderColor: '#058005',
                            label: {
                                borderColor: '#058005',
                                style: {
                                    color: '#fff',
                                    background: '#058005'
                                },
                                text: 'Expected FG'
                            }
                        }
                    ]
                },
                tooltip: {
                    x: {
                        format: "HH:mm",
                    },
                },
                colors: [
                    "#2E93fA",
                    "#DF0000",
                ],
            },
            series: [
                {
                    name: "Gravity",
                    data: this.state.gravity_data,
                },
                {
                    name: "Temperature",
                    data: this.state.temperature_data,
                }
            ],
        };

        return (
            <div className="tilt-chart">
                <Chart
                    options={graph.options}
                    series={graph.series}
                    type="line"
                    width="100%"
                    height="100%"
                />
            </div>
        );
    }

    private async refreshTiltData() {
        const url = `${window.location.protocol}//${window.location.host}/tilt/${this.props.color}?from=${this.state.from}&to=${this.state.to}`;

        let response =
            await fetch(url, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

        let readings: Reading[] = await response.json();

        let grav_data = readings.map(({at, gravity}) => (
            [
                new Date(at).getTime(),
                gravity,
            ] as ChartDatum
        ));

        let temp_data = readings.map(({at, fahrenheit}) => (
            [
                new Date(at).getTime(),
                fahrenheit
            ] as ChartDatum
        ));

        this.setState({
            gravity_data: accumulateReadings(grav_data),
            temperature_data: accumulateReadings(temp_data).map(([at, temp]) => [at, Math.round((temp - 32) * 5 / 9)])
        });
    }
}

function accumulateReadings(data: ChartDatum[]): ChartDatum[] {
    let initialAcc =
        {
        currentPeriod: null as null | number,
        currentSum: 0,
        currentCount: 0,
        data: [] as ChartDatum[],
    };

    let newAcc = data.reduce(({currentPeriod, currentSum, currentCount, data}, [x, y]) => {
            let period = ((x / 900_000) << 0) * 900_000;

            if (currentPeriod !== null && currentPeriod !== period) {

                // In-place mutation, yeuch
                data.push([ period, (currentSum / currentCount) << 0 ]);

                return (
                    {
                        currentPeriod: period,
                        currentSum: y,
                        currentCount: 1,
                        data,
                    }
                );
            }
            else {
                return (
                    {
                        currentPeriod: period,
                        currentSum: currentSum + y,
                        currentCount: currentCount + 1,
                        data,
                    }
                );
            }
        },
        initialAcc
    );

    if (newAcc.currentPeriod !== null && newAcc.currentCount > 0) {
        newAcc.data.push([
            newAcc.currentPeriod,
            (newAcc.currentSum / newAcc.currentCount) << 0
        ]);
    }

    return newAcc.data;
}

import * as React from "react";
import Chart from "react-apexcharts";

export interface TiltProps {
    color: string;
}

export interface TiltState {
    data: ChartDatum[],
}

export interface ChartDatum {
    x: number,
    y: number,
}

export interface Reading {
    at: string,
    fahrenheit: number,
    gravity: number,
}

export class Tilt extends React.Component<TiltProps, TiltState> {
    constructor(props: TiltProps) {
        super(props);
        this.state = {
            data: [],
        };
        this.refreshTiltData();
    }

    render() {
        let graph = {
            options: {
                chart: {
                    id: "basic-bar"
                },
                stroke: {
                    curve: 'straight',
                    width: 1,
                },
                xaxis: {
                    type: 'datetime'
                },
                yaxis: {
                    min: 1000,
                },
                annotations: {
                    yaxis: [
                        {
                            y: 1007,
                            borderColor: '#00E396',
                            label: {
                                borderColor: '#00E396',
                                style: {
                                    color: '#fff',
                                    background: '#00E396'
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
            },
            series: [
                {
                    name: "Gravity",
                    data: this.state.data,
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
        const url = `${window.location.protocol}//${window.location.host}/tilt/${this.props.color}/all`;

        let response =
            await fetch(url, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

        let data1: Reading[] = await response.json();

        let data2 = data1.map(({at, gravity}) => (
            {
                x: new Date(at + ":00").getTime(),
                y: gravity,
            }
        ));

        let initialAcc =
            {
                currentPeriod: null as null | number,
                currentSum: 0,
                currentCount: 0,
                data: [] as ChartDatum[],
            };

        let newAcc = data2.reduce(({currentPeriod, currentSum, currentCount, data}, {x, y}) => {
            let period = ((x / 900_000) << 0) * 900_000;

            if (currentPeriod !== null && currentPeriod !== period) {
                let newPoint = { x: period, y: (currentSum / currentCount) << 0, };

                // In-place mutation, yeuch
                data.push(newPoint);

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
            let newPoint = { x: newAcc.currentPeriod, y: (newAcc.currentSum / newAcc.currentCount) << 0, };
            newAcc.data.push(newPoint);
        }

        console.log(newAcc.data);

        this.setState({
            data: newAcc.data
        });
    }
}

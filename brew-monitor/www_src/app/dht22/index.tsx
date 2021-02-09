import * as React from "react";
import Chart from "react-apexcharts";

function useQuery(): URLSearchParams {
  return new URLSearchParams(window.location.search);
}

export interface DHT22Props {
    alias: string;
}

export interface DHT22State {
    temperature_data: ChartDatum[],
    humidity_data: ChartDatum[],
    from: string | null,
    to: string | null;
}

type ChartDatum = [number, number];

export interface Reading {
    at: string,
    temp: number,
    humidity: number,
}

export class DHT22 extends React.Component<DHT22Props, DHT22State> {
    constructor(props: DHT22Props) {
        let query = useQuery();

        super(props);
        this.state = {
            temperature_data: [],
            humidity_data: [],
            from: query.get("from"),
            to: query.get("to"),
        };
        this.refreshDHT22Data();
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
                        name: "Temperature",
                    },
                    {
                        name: "Humidity",
                        opposite: true,
                    }
                ],
                tooltip: {
                    x: {
                        format: "HH:mm",
                    },
                },
                colors: [
                    "#DF0000",
                    "#2E93fA",
                ],
            },
            series: [
                {
                    name: "Temperature",
                    data: this.state.temperature_data,
                },
                {
                    name: "Humidity",
                    data: this.state.humidity_data,
                }
            ],
        };

        return (
            <div className="dht22-chart">
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

    private async refreshDHT22Data() {
        const url = `${window.location.protocol}//${window.location.host}/dht22/${this.props.alias}/readings?from=${this.state.from}&to=${this.state.to}`;

        let response =
            await fetch(url, {
                method: "GET",
                headers: {
                    "Content-Type": "application/json",
                },
            });

        let readings: Reading[] = await response.json();

        let temp_data = readings.map(({at, temp}) => (
            [
                new Date(at).getTime(),
                temp / 100,
            ] as ChartDatum
        ));

        let humidity_data = readings.map(({at, humidity}) => (
            [
                new Date(at).getTime(),
                humidity / 100
            ] as ChartDatum
        ));

        this.setState({
            temperature_data: accumulateReadings(temp_data),
            humidity_data: accumulateReadings(humidity_data),
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

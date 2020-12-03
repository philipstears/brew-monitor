export type Notification
    = Status1Notification
    | Status2Notification
    | TempNotification
    ;

export interface Status1Notification {
    type: "Status1";
    data: Status1Data;
}

export interface Status1Data {
    heat_active: boolean;
    pump_active: boolean;
}

export interface Status2Notification {
    type: "Status2";
    data: Status2Data;
}

export interface Status2Data {
}

export interface TempNotification {
    type: "Temp";
    data: TempData;
}

export interface TempData {
    desired: number;
    current: number;
}


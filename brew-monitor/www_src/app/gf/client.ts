import * as Proto from "./types";

export class Client {
    private commandUrl: string;
    private recipeUrl: string;

    constructor(baseUrl: string) {
        this.commandUrl = `${baseUrl}/command`;
        this.recipeUrl  = `${baseUrl}/recipe`;
    }

    async incrementTargetTemperature() {
        await this.command({
            type: "IncrementTargetTemperature",
        });
    }

    async decrementTargetTemperature() {
        await this.command({
            type: "DecrementTargetTemperature",
        });
    }

    async toggleHeat() {
        await this.command({
            type: "ToggleHeatActive",
        });
    }

    async togglePump() {
        await this.command({
            type: "TogglePumpActive",
        });
    }

    async sendRecipe(recipe: Proto.Recipe) {
        await fetch(this.recipeUrl, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(recipe),
        });
    }

    async updateActiveTimer(spec: { minutes: number, seconds: number }) {
        await this.command({
            type: "UpdateActiveTimer",
            data: {
                type: "MinutesSeconds",
                data: [0, 1],
            }
        });
    }

    async skipToInteraction(interaction: Proto.InteractionCode) {
        await this.command({
            type: "SkipToInteraction",
            data: interaction,
        });
    }

    async skipToStep(stepNumber: number, skipRamp: boolean) {
        await this.command({
            type: "SkipToStep",
            data: {
                step_number: stepNumber,
                can_edit_minutes: 0,
                time_left_minutes: 0,
                time_left_seconds: 0,
                skip_ramp: skipRamp,
                disable_add_grain: true
            }
        });
    }

    async cancelSession() {
        await this.command({
            type: "Disconnect",
            data: {
                type: "CancelSession",
            }
        });
    }

    async pressSet() {
        await this.command({
            type: "PressSet",
        });
    }

    async dismissAlert() {
        await this.command({
            type: "DismissAlert",
        });
    }

    private async command(command: any) {
        await fetch(this.commandUrl, {
            method: "POST",
            headers: {
                "Content-Type": "application/json",
            },
            body: JSON.stringify(command),
        });
    }
}

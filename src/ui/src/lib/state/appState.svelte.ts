import { tick } from "svelte";

export class AppState {
  isLoading = $state(false);
  statusMessage = $state("");
  isError = $state(false);
  lastSuccessPath = $state<string | null>(null);
  accentColor = $state("#3b82f6"); // Default blue-500
  
  private statusTimeout: any = null;

  showStatus(message: string, error: boolean = false, path: string | null = null, duration: number = 6000) {
    if (this.statusTimeout) clearTimeout(this.statusTimeout);
    
    this.statusMessage = message;
    this.isError = error;
    this.isLoading = false;
    this.lastSuccessPath = path;

    if (duration > 0) {
      this.statusTimeout = setTimeout(() => {
        this.statusMessage = "";
        this.isError = false;
        this.lastSuccessPath = null;
      }, duration);
    }
  }

  startLoading(message: string = "Processing...") {
    if (this.statusTimeout) clearTimeout(this.statusTimeout);
    this.isLoading = true;
    this.statusMessage = message;
    this.isError = false;
    this.lastSuccessPath = null;
  }
}

export const appState = new AppState();

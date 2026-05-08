const state = $state({
  isLoading: false,
  statusMessage: "",
  isError: false,
  lastSuccessPath: null as string | null,
  accentColor: "#3b82f6",
  
  statusTimeout: null as any,

  showStatus(message: string, error: boolean = false, path: string | null = null, duration: number = 6000) {
    if (state.statusTimeout) clearTimeout(state.statusTimeout);
    
    state.statusMessage = message;
    state.isError = error;
    state.isLoading = false;
    state.lastSuccessPath = path;

    if (duration > 0) {
      state.statusTimeout = setTimeout(() => {
        state.statusMessage = "";
        state.isError = false;
        state.lastSuccessPath = null;
      }, duration);
    }
  },

  startLoading(message: string = "Processing...") {
    if (state.statusTimeout) clearTimeout(state.statusTimeout);
    state.isLoading = true;
    state.statusMessage = message;
    state.isError = false;
    state.lastSuccessPath = null;
  }
});

export const appState = state;

if (typeof window !== "undefined") {
  (window as any).__PINNACLE_APP_STATE__ = appState;
}

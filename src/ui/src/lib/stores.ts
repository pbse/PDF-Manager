import { writable } from 'svelte/store';

export const isLoading = writable<boolean>(false);
export const statusMessage = writable<string>('');
export const isError = writable<boolean>(false);

export function showStatus(message: string, error: boolean = false, duration: number = 4000) {
    statusMessage.set(message);
    isError.set(error);
    isLoading.set(false); // Ensure loading is off when showing status

    if (duration > 0) {
        setTimeout(() => {
            statusMessage.set('');
            isError.set(false);
        }, duration);
    }
}

export function startLoading(message: string = 'Processing...') {
    isLoading.set(true);
    statusMessage.set(message);
    isError.set(false);
}
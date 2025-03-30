<script lang="ts">
    import { isLoading, statusMessage, isError } from '$lib/stores';
</script>

{#if $isLoading || $statusMessage}
    <div class="status-bar" class:error={$isError} class:loading={$isLoading}>
        {#if $isLoading}
            <span class="spinner">⏳</span> <!-- Basic spinner -->
        {/if}
        {$statusMessage}
    </div>
{/if}

<style>
    .status-bar {
        position: fixed;
        bottom: 10px;
        left: 50%;
        transform: translateX(-50%);
        padding: 8px 15px;
        border-radius: 5px;
        background-color: #eee;
        color: #333;
        box-shadow: 0 2px 5px rgba(0,0,0,0.2);
        z-index: 1000;
        font-size: 0.9em;
        display: flex;
        align-items: center;
        gap: 8px;
    }
    .status-bar.loading {
        background-color: #d1e7fd;
        color: #0c5460;
    }
    .status-bar.error {
        background-color: #f8d7da;
        color: #721c24;
    }
    .spinner {
        display: inline-block;
        animation: spin 1s linear infinite;
    }
    @keyframes spin {
        from { transform: rotate(0deg); }
        to { transform: rotate(360deg); }
    }
</style>
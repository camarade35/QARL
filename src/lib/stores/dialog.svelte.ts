// src/lib/stores/dialog.svelte.ts
export const dialogState = $state({
  isOpen: false,
  type: 'alert' as 'alert' | 'confirm' | 'prompt' | 'update',
  title: '',
  message: '',
  notes: '',
  notesLabel: '',
  emptyNotesMessage: '',
  variant: 'info' as 'info' | 'warning' | 'error' | 'success',
  okLabel: '',
  cancelLabel: '',
  inputValue: '',
  resolve: (value: any) => {}
});

function showDialog(opts: Partial<typeof dialogState>): Promise<any> {
  return new Promise((resolve) => {
    Object.assign(dialogState, {
      isOpen: true,
      type: 'alert',
      title: '',
      message: '',
      notes: '',
      notesLabel: '',
      emptyNotesMessage: '',
      variant: 'info',
      okLabel: '',
      cancelLabel: '',
      inputValue: '',
      ...opts,
      resolve
    });
  });
}

export const alert = (message: string, title = '', variant: 'info'|'warning'|'error'|'success' = 'info') => 
  showDialog({ type: 'alert', message, title, variant });

export const confirm = (message: string, title = '', variant: 'info'|'warning'|'error'|'success' = 'warning', okLabel = '', cancelLabel = '') => 
  showDialog({ type: 'confirm', message, title, variant, okLabel, cancelLabel });

export const updatePrompt = (options: {
  title: string;
  message: string;
  notes: string;
  notesLabel: string;
  emptyNotesMessage: string;
  installLabel: string;
  ignoreLabel: string;
}): Promise<boolean> =>
  showDialog({
    type: 'update',
    title: options.title,
    message: options.message,
    notes: options.notes,
    notesLabel: options.notesLabel,
    emptyNotesMessage: options.emptyNotesMessage,
    okLabel: options.installLabel,
    cancelLabel: options.ignoreLabel,
    variant: 'info'
  });

export const prompt = (message: string, defaultValue = '', title = '') => 
  showDialog({ type: 'prompt', message, title, inputValue: defaultValue });
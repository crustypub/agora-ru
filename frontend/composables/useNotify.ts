export const useNotify = () => {
  const toast = useToast()

  const success = (title: string, description?: string) => {
    toast.add({
      title,
      description,
      color: 'green',
      icon: 'i-heroicons-check-circle-20-solid',
    })
  }

  const error = (title: string, description?: string) => {
    toast.add({
      title,
      description,
      color: 'red',
      icon: 'i-heroicons-exclamation-triangle-20-solid',
    })
  }

  const warn = (title: string, description?: string) => {
    toast.add({
      title,
      description,
      color: 'yellow',
      icon: 'i-heroicons-exclamation-circle-20-solid',
    })
  }

  return {
    success,
    error,
    warn,
  }
}

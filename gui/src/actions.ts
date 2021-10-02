export function longpress(
  node: Element,
  threshold = 500
): { destroy: () => void } {
  const handle_mousedown = () => {
    const timeout = setTimeout(
      () => node.dispatchEvent(new CustomEvent('longpress')),
      threshold
    )

    const cancel = () => {
      clearTimeout(timeout)
      node.removeEventListener('mousemove', cancel)
      node.removeEventListener('mouseup', cancel)
    }

    node.addEventListener('mousemove', cancel)
    node.addEventListener('mouseup', cancel)
  }

  node.addEventListener('mousedown', handle_mousedown)

  return {
    destroy() {
      node.removeEventListener('mousedown', handle_mousedown)
    },
  }
}

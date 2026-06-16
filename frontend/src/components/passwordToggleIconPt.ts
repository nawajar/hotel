// Passthrough for PrimeVue's unstyled Password component: it ships no
// positioning CSS for the show/hide icon, so without this it renders as a
// plain block element below the input instead of overlaid inside it.
export const passwordToggleIconPt = {
  root: { class: "relative" },
  maskIcon: { class: "absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 cursor-pointer" },
  unmaskIcon: { class: "absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 cursor-pointer" },
};

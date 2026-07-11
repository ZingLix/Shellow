export function Iphone({ src, frame = "dark", className = "" }) {
  return (
    <div className={`magic-iphone magic-iphone-${frame} ${className}`}>
      <img src={src} alt="" />
      <div className="magic-iphone-island" aria-hidden="true" />
    </div>
  );
}

import Foundation
import MetalKit
import SwiftUI

private final class TerminalMetalHostView: MTKView {
    var onLayout: ((TerminalMetalHostView) -> Void)?

    override func layoutSubviews() {
        super.layoutSubviews()
        onLayout?(self)
    }
}

struct TerminalMetalGridSurface: UIViewRepresentable {
    @Environment(\.colorScheme) private var colorScheme

    static var isAvailable: Bool {
        MTLCreateSystemDefaultDevice() != nil
    }

    let viewportFirstRow: Int
    let viewportRowCount: Int
    let renderTick: Int
    let rendererOverlayJSON: String
    let attachRendererSurface: (UInt64, Int, Int) -> Void
    let setRendererOverlay: (String) -> Void
    let renderRendererSurface: (Int, Int, Int, Int) -> Bool
    let detachRendererSurface: () -> Void

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> MTKView {
        let view = TerminalMetalHostView(frame: .zero, device: MTLCreateSystemDefaultDevice())
        view.colorPixelFormat = .bgra8Unorm
        view.framebufferOnly = true
        view.isPaused = true
        view.enableSetNeedsDisplay = true
        view.autoResizeDrawable = true
        view.clearColor = metalClearColor(for: colorScheme)
        view.delegate = context.coordinator
        view.onLayout = { [weak coordinator = context.coordinator] view in
            coordinator?.layoutChanged(view: view)
        }
        context.coordinator.configure(device: view.device)
        updateUIView(view, context: context)
        return view
    }

    func updateUIView(_ view: MTKView, context: Context) {
        view.clearColor = metalClearColor(for: colorScheme)
        context.coordinator.update(
            TerminalMetalGridState(
                viewportFirstRow: viewportFirstRow,
                viewportRowCount: viewportRowCount,
                renderTick: renderTick
            )
        )
        context.coordinator.attachIfNeeded(
            view: view,
            attach: attachRendererSurface,
            setOverlay: setRendererOverlay,
            render: renderRendererSurface,
            detach: detachRendererSurface
        )
        context.coordinator.updateOverlayJSON(rendererOverlayJSON)
        if !context.coordinator.renderWithRustSurfaceIfPossible(view: view) {
            view.setNeedsDisplay()
        }
    }

    private func metalClearColor(for colorScheme: ColorScheme) -> MTLClearColor {
        switch colorScheme {
        case .dark:
            MTLClearColor(red: 0.05, green: 0.06, blue: 0.06, alpha: 1)
        case .light:
            MTLClearColor(red: 0.97, green: 0.98, blue: 0.96, alpha: 1)
        @unknown default:
            MTLClearColor(red: 0.05, green: 0.06, blue: 0.06, alpha: 1)
        }
    }

    static func dismantleUIView(_ view: MTKView, coordinator: Coordinator) {
        coordinator.detachSurface()
        (view as? TerminalMetalHostView)?.onLayout = nil
        view.delegate = nil
    }

    final class Coordinator: NSObject, MTKViewDelegate {
        private var commandQueue: MTLCommandQueue?
        private var state: TerminalMetalGridState?
        private var lastSurfaceSignature: String?
        private var overlayJSON = #"{"ranges":[]}"#
        private var lastAppliedOverlayJSON: String?
        private var attachRendererSurface: ((UInt64, Int, Int) -> Void)?
        private var setRendererOverlay: ((String) -> Void)?
        private var renderRendererSurface: ((Int, Int, Int, Int) -> Bool)?
        private var detachRendererSurface: (() -> Void)?

        func configure(device: MTLDevice?) {
            commandQueue = device?.makeCommandQueue()
        }

        fileprivate func attachIfNeeded(
            view: MTKView,
            attach: @escaping (UInt64, Int, Int) -> Void,
            setOverlay: @escaping (String) -> Void,
            render: @escaping (Int, Int, Int, Int) -> Bool,
            detach: @escaping () -> Void
        ) {
            attachRendererSurface = attach
            setRendererOverlay = setOverlay
            renderRendererSurface = render
            detachRendererSurface = detach
            attachIfReady(view: view)
        }

        fileprivate func updateOverlayJSON(_ overlayJSON: String) {
            self.overlayJSON = overlayJSON
        }

        fileprivate func layoutChanged(view: MTKView) {
            attachIfReady(view: view)
            if !renderWithRustSurfaceIfPossible(view: view) {
                view.setNeedsDisplay()
            }
        }

        fileprivate func attachIfReady(view: MTKView) {
            let size = surfacePixelSize(for: view)
            let width = size.width
            let height = size.height
            guard width > 0, height > 0 else { return }
            guard let attachRendererSurface else { return }

            let rawHandle = UInt64(UInt(bitPattern: Unmanaged.passUnretained(view.layer).toOpaque()))
            let signature = "\(rawHandle):\(width)x\(height)"
            guard signature != lastSurfaceSignature else { return }

            lastSurfaceSignature = signature
            attachRendererSurface(rawHandle, width, height)
        }

        fileprivate func detachSurface() {
            guard lastSurfaceSignature != nil else { return }
            detachRendererSurface?()
            lastSurfaceSignature = nil
            lastAppliedOverlayJSON = nil
        }

        fileprivate func update(_ state: TerminalMetalGridState) {
            self.state = state
        }

        func mtkView(_ view: MTKView, drawableSizeWillChange size: CGSize) {
            lastSurfaceSignature = nil
            attachIfReady(view: view)
        }

        func draw(in view: MTKView) {
            attachIfReady(view: view)
            if renderWithRustSurfaceIfPossible(view: view) {
                return
            }

            clear(view: view)
        }

        private func clear(view: MTKView) {
            guard
                let commandQueue,
                let descriptor = view.currentRenderPassDescriptor,
                let drawable = view.currentDrawable
            else {
                return
            }

            descriptor.colorAttachments[0].loadAction = .clear
            descriptor.colorAttachments[0].clearColor = view.clearColor
            let commandBuffer = commandQueue.makeCommandBuffer()
            let encoder = commandBuffer?.makeRenderCommandEncoder(descriptor: descriptor)
            encoder?.endEncoding()
            commandBuffer?.present(drawable)
            commandBuffer?.commit()
        }

        fileprivate func renderWithRustSurfaceIfPossible(view: MTKView) -> Bool {
            guard let state else { return false }
            guard lastSurfaceSignature != nil, let renderRendererSurface else { return false }
            let size = surfacePixelSize(for: view)
            let width = size.width
            let height = size.height
            guard width > 0, height > 0 else { return false }
            if overlayJSON != lastAppliedOverlayJSON {
                setRendererOverlay?(overlayJSON)
                lastAppliedOverlayJSON = overlayJSON
            }
            return renderRendererSurface(
                width,
                height,
                state.viewportFirstRow,
                state.viewportRowCount
            )
        }

        private func surfacePixelSize(for view: MTKView) -> (width: Int, height: Int) {
            var width = Int(view.drawableSize.width.rounded(.toNearestOrAwayFromZero))
            var height = Int(view.drawableSize.height.rounded(.toNearestOrAwayFromZero))
            if width <= 0 || height <= 0 {
                width = Int((view.bounds.width * view.contentScaleFactor).rounded(.toNearestOrAwayFromZero))
                height = Int((view.bounds.height * view.contentScaleFactor).rounded(.toNearestOrAwayFromZero))
            }
            return (width, height)
        }
    }
}

private struct TerminalMetalGridState {
    let viewportFirstRow: Int
    let viewportRowCount: Int
    let renderTick: Int
}

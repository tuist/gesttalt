import SwiftUI

struct ContentView: View {
    @State private var name = "Swift"
    private let bridge = RustBridge()

    var body: some View {
        NavigationStack {
            Form {
                Section {
                    TextField("Name", text: $name)
                        .textInputAutocapitalization(.words)
                }

                Section {
                    Text(bridge.greeting(name: name))
                    Text("Lattice score: \(bridge.latticeScore(seed: Int32(name.count)))")
                        .font(.headline)
                }
            }
            .navigationTitle("Gesttalt")
        }
    }
}

#Preview {
    ContentView()
}


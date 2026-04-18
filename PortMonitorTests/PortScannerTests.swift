import XCTest
@testable import PortMonitor

final class PortScannerTests: XCTestCase {

    func testParsesMultiplePorts() {
        let output = """
        COMMAND     PID   USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
        node       1234   alice  22u  IPv6  12345      0t0  TCP *:3000 (LISTEN)
        vite       2345   alice  18u  IPv4  12346      0t0  TCP 127.0.0.1:8080 (LISTEN)
        postgres   9876   _pg    10u  IPv6  12347      0t0  TCP *:5432 (LISTEN)
        """
        let entries = PortScanner.parse(output)
        XCTAssertEqual(entries.count, 3)
        XCTAssertEqual(entries[0].port, 3000)
        XCTAssertEqual(entries[0].processName, "node")
        XCTAssertEqual(entries[0].pid, 1234)
        XCTAssertEqual(entries[0].user, "alice")
        XCTAssertEqual(entries[1].port, 5432)
        XCTAssertEqual(entries[1].user, "_pg")
        XCTAssertEqual(entries[2].port, 8080)
    }

    func testSkipsHeaderLine() {
        let output = "COMMAND     PID   USER   FD   TYPE DEVICE SIZE/OFF NODE NAME\n"
        let entries = PortScanner.parse(output)
        XCTAssertTrue(entries.isEmpty)
    }

    func testDeduplicatesSamePort() {
        let output = """
        COMMAND   PID   USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
        node     1234   user   22u  IPv6  12345      0t0  TCP *:3000 (LISTEN)
        node     1234   user   23u  IPv4  12346      0t0  TCP 0.0.0.0:3000 (LISTEN)
        """
        let entries = PortScanner.parse(output)
        XCTAssertEqual(entries.count, 1)
        XCTAssertEqual(entries[0].port, 3000)
    }

    func testSkipsMalformedLines() {
        let output = """
        COMMAND   PID   USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
        badline
        node     1234   user   22u  IPv6  12345      0t0  TCP *:3000 (LISTEN)
        """
        let entries = PortScanner.parse(output)
        XCTAssertEqual(entries.count, 1)
    }

    func testSkipsNonNumericPort() {
        let output = """
        COMMAND   PID   USER   FD   TYPE DEVICE SIZE/OFF NODE NAME
        node     1234   user   22u  IPv6  12345      0t0  TCP *:http (LISTEN)
        node     2345   user   23u  IPv6  12346      0t0  TCP *:3000 (LISTEN)
        """
        let entries = PortScanner.parse(output)
        XCTAssertEqual(entries.count, 1)
        XCTAssertEqual(entries[0].port, 3000)
    }
}

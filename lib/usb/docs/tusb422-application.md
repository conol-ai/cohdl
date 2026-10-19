# TUSB422 application contract

Source: [TI SLLSEW6C](https://www.ti.com/lit/ds/symlink/tusb422.pdf), June 2018, including the November 2025 orderable addendum; retrieved unchanged on 2026-10-17 as `ti-tusb422.pdf`. SHA-256: `36c16474811dad1036722e164b7b4647aba669acc966fb342693ac41abc9c677`. Page numbers below are one-based PDF pages.

## Identity, host interface and support circuit

Exact order codes: **TUSB422IYFPR** and **TUSB422IYFPT**, YFP nine-ball DSBGA, differing reel quantities (p71). The pin-page heading says YFF, but the exact order codes and part-specific drawing identify **YFP0009-C01** (pp68-71); no generic YFF footprint substitution is implied.

An external MCU runs the TCPM; this package contains no firmware. I2C is fixed at **0x20, 7-bit**, up to 1 MHz (pp12,19). SCL and SDA are open-drain bidirectional pins; INT_N is an open-drain output. Provide pull-ups and wait for INT_N initialization before register access (pp3,65). CoHDL roles do not encode open-drain electrical behavior.

Normal VDD is 2.7-5.5 V; I2C pull-up rail 1.65-3.6 V. **VDD must exceed 3 V for 3.3 V I2C operation** (pp4,7). Figure 76 uses 100 nF at VDD. CC1/CC2 each require 200-450 pF external shunt capacitance, with 220 pF in the example. Pull-up values must match bus capacitance and speed; Table 73's 1.5 kohm at 400 kHz/150 pF is an example, not a universal choice (pp7,63-64).

VBUSIN operates at 0-24 V and includes measurement/discharge functions; it is not the load-current path. VCONN is a separate 2.7-5.5 V input; it may float if unused. Otherwise establish a stable supply before enabling its switch and meet the source's capacitance/current/voltage-drop requirements (pp3-4,7,64). The declared power-input role does not simulate discharge, leakage or power sequencing.

## Protocol and passive-monitor limits

The PHY handles automatic GoodCRC (p11), is disabled while unattached, and follows configured orientation/role (p12). **PD 3.0 Get Source Capabilities Extended is unsupported:** the device neither acknowledges nor raises an RX alert for it (p12). Do not present this part as a complete PD 3.0 analyzer.

With VDD absent it applies **Rd to both CC pins**; registers are inaccessible in dead-battery mode (p18). Receiving packets is therefore not proof of passive capture. A passive-monitor design needs separate evidence for all power/reset states, loading, reply suppression and orientation/VCONN continuity. No such architecture is qualified here.

## Pin and land audit

Page 3 fixes A1=CC2, A2=VBUSIN, A3=VDD; B1=VCONN, B2=INT_N, B3=SCL; C1=CC1, C2=GND, C3=SDA. Page 69 supplies nine **0.23 mm diameter copper** circles on **0.4 mm pitch**, viewed from the board/component top. Do not mirror the package's bottom-view drawing. No EP exists.

The library uses the preferred NSMD copper interpretation. Solder-mask openings, stencil and assembly escape constraints remain board/process decisions (pp69-70). The 2.7 mm courtyard and A1 ink dot are library-authored and include clearance; they are not TI dimensions. This definition does not establish JLCPCB stock or manufacturing readiness. Manufacturer PDF copyright/notices remain intact; MIT covers the authored declarations and notes.

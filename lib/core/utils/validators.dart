/// Typed input validation utilities for Zettings UI forms and settings payloads.
///
/// Ensures inputs comply with system boundaries before dispatching to D-Bus IPC
/// or writing to persistent storage.
abstract final class Validators {
  /// Validates a master audio volume ratio (0.0 .. 1.0).
  static String? validateAudioVolume(double? volume) {
    if (volume == null) {
      return 'Volume cannot be null';
    }
    if (volume.isNaN || volume.isInfinite) {
      return 'Volume must be a finite number';
    }
    if (volume < 0.0 || volume > 1.0) {
      return 'Volume must be between 0.0 and 1.0';
    }
    return null;
  }

  /// Validates display scaling factor (1.0 .. 2.5).
  static String? validateDisplayScaling(double? factor) {
    if (factor == null) {
      return 'Scaling factor cannot be null';
    }
    if (factor.isNaN || factor.isInfinite) {
      return 'Scaling factor must be a finite number';
    }
    if (factor < 1.0 || factor > 2.5) {
      return 'Scaling factor must be between 1.0 (100%) and 2.5 (250%)';
    }
    return null;
  }

  /// Validates night light color temperature (1000K .. 10000K).
  static String? validateColorTemperature(int? tempKelvin) {
    if (tempKelvin == null) {
      return 'Color temperature cannot be null';
    }
    if (tempKelvin < 1000 || tempKelvin > 10000) {
      return 'Color temperature must be between 1000K and 10000K';
    }
    return null;
  }

  /// Validates an IEEE 802.11 Wi-Fi SSID (1 .. 32 bytes).
  static String? validateWifiSsid(String? ssid) {
    if (ssid == null || ssid.isEmpty) {
      return 'Wi-Fi network name cannot be empty';
    }
    if (ssid.length > 32) {
      return 'Wi-Fi network name cannot exceed 32 characters';
    }
    return null;
  }

  /// Validates a WPA/WPA2 Wi-Fi password (8 .. 63 characters).
  static String? validateWifiPassword(String? password) {
    if (password == null || password.isEmpty) {
      return 'Password cannot be empty';
    }
    if (password.length < 8) {
      return 'Wi-Fi password must be at least 8 characters';
    }
    if (password.length > 63) {
      return 'Wi-Fi password cannot exceed 63 characters';
    }
    return null;
  }

  /// Validates an IPv4 address format (e.g. 192.168.1.1).
  static String? validateIpv4Address(String? address) {
    if (address == null || address.isEmpty) {
      return 'IP address cannot be empty';
    }
    final parts = address.split('.');
    if (parts.length != 4) {
      return 'IPv4 address must contain exactly 4 octets';
    }
    for (final part in parts) {
      final value = int.tryParse(part);
      if (value == null || value < 0 || value > 255) {
        return 'Each octet must be an integer between 0 and 255';
      }
      if (part.length > 1 && part.startsWith('0')) {
        return 'Octets cannot contain leading zeros';
      }
    }
    return null;
  }

  /// Validates a Linux username according to POSIX conventions.
  static String? validateUsername(String? username) {
    if (username == null || username.isEmpty) {
      return 'Username cannot be empty';
    }
    if (username.length > 32) {
      return 'Username cannot exceed 32 characters';
    }
    final firstChar = username.codeUnitAt(0);
    final isLower = firstChar >= 0x61 && firstChar <= 0x7A;
    final isUnderscore = firstChar == 0x5F;
    if (!isLower && !isUnderscore) {
      return 'Username must start with a lowercase letter or underscore';
    }
    for (var i = 1; i < username.length; i++) {
      final c = username.codeUnitAt(i);
      final charLower = c >= 0x61 && c <= 0x7A;
      final charDigit = c >= 0x30 && c <= 0x39;
      final charSpecial = c == 0x5F || c == 0x2D; // '_' or '-'
      if (!charLower && !charDigit && !charSpecial) {
        return 'Username contains invalid characters (use lowercase, digits, _, -)';
      }
    }
    return null;
  }

  /// Validates a power profile name.
  static String? validatePowerProfile(String? profile) {
    if (profile == null || profile.trim().isEmpty) {
      return 'Power profile cannot be empty';
    }
    const valid = {'performance', 'balanced', 'power-saver'};
    if (!valid.contains(profile.trim().toLowerCase())) {
      return 'Unknown power profile: $profile';
    }
    return null;
  }
}

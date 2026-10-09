import 'package:zettings/core/errors/app_error.dart';
import 'package:zettings/core/ipc/zettings_ipc_client.dart';
import 'package:zettings/core/models/audio_status.dart';
import 'package:zettings/core/platform/capability.dart';
import 'package:zettings/core/services/system_service.dart';

abstract class AudioServiceProvider {
  Future<AudioStatus> getStatus();
  Future<void> setVolume(double volume);
  Future<void> setMuted(bool muted);
}

class IpcAudioService implements AudioServiceProvider, SystemService {
  IpcAudioService(this._client);

  final ZettingsIpcClient _client;

  @override
  String get name => 'audio';

  @override
  Future<SystemCapability> probe() async {
    try {
      final caps = await _client.getCapabilities();
      final match = caps.where((c) => c.name == name);
      if (match.isEmpty) {
        return const SystemCapability(
          name: 'audio',
          availability: ServiceAvailability.unavailable,
          reason: 'Audio service not registered in backend',
        );
      }
      final raw = match.first;
      final availability = switch (raw.availability) {
        'available' => ServiceAvailability.available,
        'unsupported' => ServiceAvailability.unsupported,
        'permissionDenied' => ServiceAvailability.permissionDenied,
        _ => ServiceAvailability.unavailable,
      };
      return SystemCapability(
        name: name,
        availability: availability,
        reason: raw.reason.isEmpty ? 'Audio subsystem active' : raw.reason,
      );
    } on AppError catch (e) {
      return SystemCapability(
        name: name,
        availability: e.type == AppErrorType.permissionDenied
            ? ServiceAvailability.permissionDenied
            : ServiceAvailability.unavailable,
        reason: e.message,
      );
    } catch (e) {
      return SystemCapability(
        name: name,
        availability: ServiceAvailability.unavailable,
        reason: '$e',
      );
    }
  }

  @override
  Future<AudioStatus> getStatus() => _client.getAudioStatus();

  @override
  Future<void> setVolume(double volume) => _client.setMasterVolume(volume);

  @override
  Future<void> setMuted(bool muted) => _client.setMasterMute(muted);
}

class MockAudioService implements AudioServiceProvider, SystemService {
  MockAudioService({double initialVolume = 0.65, bool initialMuted = false})
    : _volume = initialVolume,
      _muted = initialMuted;

  double _volume;
  bool _muted;

  @override
  String get name => 'audio';

  @override
  Future<SystemCapability> probe() async => const SystemCapability(
    name: 'audio',
    availability: ServiceAvailability.available,
    reason: 'Mock Development Backend (Preview)',
  );

  @override
  Future<AudioStatus> getStatus() async {
    return AudioStatus(
      masterVolume: _volume,
      muted: _muted,
      defaultSink: 'alsa_output.pci-0000_00_1f.3.analog-stereo',
      defaultSource: 'alsa_input.pci-0000_00_1f.3.analog-stereo',
    );
  }

  @override
  Future<void> setVolume(double volume) async {
    _volume = volume.clamp(0.0, 1.0);
  }

  @override
  Future<void> setMuted(bool muted) async {
    _muted = muted;
  }
}

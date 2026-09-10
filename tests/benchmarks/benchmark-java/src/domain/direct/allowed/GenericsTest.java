package domain.direct.allowed;

import technology.direct.dao.ProfileDAO;
import java.util.List;
import java.util.Map;

public class GenericsTest {

    public static class Repository<T> {
        private T entity;

        public T getEntity() {
            return entity;
        }
    }

    public static class Entry<K, V> {
        private K key;
        private V value;
    }

    private Repository<ProfileDAO> profileRepo;
    private List<ProfileDAO> profileList;
    private Map<String, List<ProfileDAO>> groupedProfiles;

    public <T extends ProfileDAO> void executeDao(T dao) {
        dao.getCampaignType();
    }
}
